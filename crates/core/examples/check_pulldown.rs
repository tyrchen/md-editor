use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

fn main() {
    let markdown = "# Heading\n\nParagraph with *italic* and **bold**.\n\n> Blockquote\n\n";
    let options = Options::all();
    let parser = Parser::new_ext(markdown, options);

    // Collect all events for inspection
    let events: Vec<Event> = parser.collect();

    for event in events {
        match &event {
            Event::Start(tag) => {
                println!("Start Tag: {:?}", tag);
                // Examine structure of specific tags
                match tag {
                    Tag::Paragraph => println!("  - Paragraph is a unit variant"),
                    Tag::Heading { level, .. } => {
                        println!("  - Heading is a struct variant with level: {:?}", level)
                    }
                    Tag::BlockQuote(kind) => {
                        println!("  - BlockQuote is a struct variant with kind: {:?}", kind)
                    }
                    Tag::Link {
                        dest_url, title, ..
                    } => {
                        println!("  - Link is a struct variant");
                        println!("    - dest_url: {:?}", dest_url);
                        println!("    - title: {:?}", title);
                    }
                    _ => {}
                }
            }
            Event::End(tag) => {
                println!("End Tag: {:?}", tag);
                match tag {
                    TagEnd::Paragraph => println!("  - TagEnd::Paragraph is a unit variant"),
                    TagEnd::Heading(level) => {
                        println!(
                            "  - TagEnd::Heading is a struct variant with level: {:?}",
                            level
                        )
                    }
                    TagEnd::BlockQuote(kind) => {
                        println!(
                            "  - TagEnd::BlockQuote is a struct variant with kind: {:?}",
                            kind
                        )
                    }
                    _ => {}
                }
            }
            _ => println!("Event: {:?}", event),
        }
    }
}
