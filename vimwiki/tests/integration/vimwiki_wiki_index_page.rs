use super::fixtures::VimwikiFile;
use std::convert::TryInto;
use vimwiki::{components::*, RawStr, LC};
use vimwiki_macros::*;

#[test]
fn test() {
    let _page: LC<Page> =
        RawStr::Vimwiki(&VimwikiFile::VimwikiWikiIndex.load().unwrap())
            .try_into()
            .unwrap();
    todo!();

    // {
    //     let c = &page.components[0];
    //     assert_eq!(
    //         c.component,
    //         BlockComponent::Header(Header {
    //             level: 1,
    //             text: "Vimwiki Wiki".to_string(),
    //             centered: false,
    //         })
    //     );
    //     assert_eq!(c.region, Region::from(((0, 0), (0, 16))));
    // }

    // {
    //     let c = &page.components[1];
    //     assert_eq!(c.component, BlockComponent::BlankLine);
    //     assert_eq!(c.region, Region::from(((1, 0), (1, 0))));
    // }

    // {
    //     let c = &page.components[2];
    //     match &c.component {
    //         BlockComponent::Paragraph(Paragraph { content }) => {
    //             assert_eq!(content.len(), 1);
    //             match &content[0].component {
    //                 InlineComponent::DecoratedText(DecoratedText {
    //                     contents,
    //                     decoration,
    //                 }) => {
    //                     assert_eq!(contents.len(), 1);
    //                     assert_eq!(
    //                         contents[0].component,
    //                         DecoratedTextContent::Text(
    //                             "Welcome to the Vimwiki wiki!".to_string()
    //                         )
    //                     );
    //                     assert_eq!(
    //                         contents[0].region,
    //                         Region::from(((2, 1), (2, 28)))
    //                     );
    //                     assert_eq!(*decoration, Decoration::Bold);
    //                 }
    //                 x => panic!("Unexpected inline component: {:?}", x),
    //             }
    //             assert_eq!(content[0].region, Region::from(((2, 0), (2, 29))));
    //         }
    //         x => panic!("Unexpected block component: {:?}", x),
    //     }
    //     assert_eq!(c.region, Region::from(((2, 0), (2, 30))));
    // }

    // {
    //     let c = &page.components[3];
    //     assert_eq!(c.component, BlockComponent::BlankLine);
    //     assert_eq!(c.region, Region::from(((3, 0), (3, 0))));
    // }

    // {
    //     let c = &page.components[4];
    //     assert_eq!(
    //         c.component,
    //         BlockComponent::Header(Header {
    //             level: 2,
    //             text: "Official Repositories".to_string(),
    //             centered: false,
    //         })
    //     );
    //     assert_eq!(c.region, Region::from(((4, 0), (4, 27))));
    // }

    // {
    //     let c = &page.components[5];
    //     assert_eq!(c.component, BlockComponent::BlankLine);
    //     assert_eq!(c.region, Region::from(((5, 0), (5, 0))));
    // }

    // {
    //     let c = &page.components[6];
    //     match &c.component {
    //         BlockComponent::Paragraph(Paragraph { content }) => {
    //             assert_eq!(content.len(), 1);
    //             assert_eq!(
    //                 content[0].component,
    //                 InlineComponent::Text(
    //                     "Here are links to the official Vimwiki repositories:"
    //                         .to_string()
    //                 )
    //             );
    //             assert_eq!(content[0].region, Region::from(((6, 0), (6, 51))));
    //         }
    //         x => panic!("Unexpected block component: {:?}", x),
    //     }
    //     assert_eq!(c.region, Region::from(((6, 0), (6, 52))));
    // }

    // {
    //     let c = &page.components[7];
    //     assert_eq!(c.component, BlockComponent::BlankLine);
    //     assert_eq!(c.region, Region::from(((7, 0), (7, 0))));
    // }

    // {
    //     let c = &page.components[8];
    //     match &c.component {
    //         BlockComponent::List(List { items }) => {
    //             assert_eq!(items.len(), 4, "Wrong number of list items");

    //             let ListItem {
    //                 item_type,
    //                 suffix,
    //                 pos,
    //                 contents,
    //             } = &items[0].component.item;
    //             assert_eq!(
    //                 item_type,
    //                 ListItemType::Unordered(UnorderedListItemType::Hyphen)
    //             );
    //             assert_eq!(suffix, ListItemSuffix::None);
    //             assert_eq!(pos, 0);

    //             assert!(
    //                 items[0].attributes.is_empty(),
    //                 "List item contains attributes: {:?}",
    //                 items[0].component.attributes
    //             );
    //             assert_eq!(items[0].region, Region::from(((8, 0), (8, 48))));
    //         }
    //         x => panic!("Unexpected block component: {:?}", x),
    //     }
    //     assert_eq!(c.region, Region::from(((8, 0), (14, 58))));
    // }
}
