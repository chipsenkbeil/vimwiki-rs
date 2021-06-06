# vimwiki wasm

Provides wasm bindings for vimwiki library.

## Usage

TODO - publish npm package and provide guidance

## Examples

```html
<!-- Note the usage of `type=module` here as this is an ES6 module -->
<script type="module">
    import init, { parse_vimwiki_str } from './vimwiki_wasm.js';

    async function run() {
        // If building as web, need to do this
        await init();

        // Load the code snippet from a dom element
        const code = document.getElementById("vimwiki-snippet");

        // Parse the code into an object
        const page = parse_vimwiki_str(code.innerText);

        // Find regions in code that contain headers and highlight them by
        // transforming that text into spans with colors
        const regions = page.descendants
          .filter(e => e.is_block())
          .map(e => e.into_block())
          .filter(e => e.is_header())
          .map(e => e.into_header().region);

        // For each header's region in the loaded text...
        Object.values(regions).forEach(region => {
            // Select the header
            const range = new Range();
            range.setStart(code.firstChild, region.offset);
            range.setEnd(code.firstChild, region.len);

            // Build a colored version
            const colored = document.createElement("span");
            colored.style.color = "red";
            colored.innerText = range.toString();

            // Swap the contents with the colored version
            range.deleteContents();
            range.insertNode(colored);
        });

        // Render vimwiki as HTML and inject into output destination
        const output = document.getElementById("vimwiki-output");
        output.insertAdjacentHTML("afterbegin", page.to_html_str());
    }

    run();
</script>
```

## Building from source

Compiling without webpack bundler:

`wasm-pack build --target web`

## License

This project is licensed under either of

Apache License, Version 2.0, (LICENSE-APACHE or
[apache-license][apache-license]) MIT license (LICENSE-MIT or
[mit-license][mit-license]) at your option.

[apache-license]: http://www.apache.org/licenses/LICENSE-2.0
[mit-license]: http://opensource.org/licenses/MIT
