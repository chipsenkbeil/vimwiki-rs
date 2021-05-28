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

        // Parse some vimwiki into an object
        const obj = parse_vimwiki_str("= my header =");

        // Entire object can be converted to JavaScript object
        console.log("vimwiki obj", obj.to_js());

        // Object can be converted into HTML to be injected into DOM
        const html_str = obj.to_html_str();

        // Load some random dom element and add the vimwiki HTML output as
        // the first node within its children
        document.getElementById("some-element").insertAdjacentHTML("afterbegin", html_str);
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
