use md_core::{Document, Editor, InlineNode, ListItem, ListType, Node};

#[test]
fn test_edit_task_item_integration() {
    // Create a document with a task list containing three items
    let mut document = Document::new();

    let items = vec![
        ListItem::task("Task 1", false),
        ListItem::task("Task 2", false),
        ListItem::task("Task 3", false),
    ];

    let list = Node::List {
        list_type: ListType::Task,
        items,
    };

    document.nodes.push(list);

    let mut editor = Editor::new(document);

    // Verify the initial state
    let doc = editor.document().borrow();
    assert_eq!(doc.nodes.len(), 1);

    match &doc.nodes[0] {
        Node::List { items, .. } => {
            assert_eq!(items.len(), 3);

            // Check second task text
            match &items[1].children[0] {
                Node::Paragraph { children } => match &children[0] {
                    InlineNode::Text(text_node) => {
                        assert_eq!(text_node.text, "Task 2");
                    }
                    _ => panic!("Expected text node"),
                },
                _ => panic!("Expected paragraph node"),
            }
        }
        _ => panic!("Expected list node"),
    }

    // Need to drop the borrow before continuing
    drop(doc);

    // Edit the middle task item
    let result = editor.edit_task_item(0, 1, "Updated Task 2");
    assert!(result.is_ok());

    // Verify the task item was edited
    let doc = editor.document().borrow();
    match &doc.nodes[0] {
        Node::List { items, .. } => match &items[1].children[0] {
            Node::Paragraph { children } => match &children[0] {
                InlineNode::Text(text_node) => {
                    assert_eq!(text_node.text, "Updated Task 2");
                }
                _ => panic!("Expected text node"),
            },
            _ => panic!("Expected paragraph node"),
        },
        _ => panic!("Expected list node"),
    }

    // Drop borrow before continuing
    drop(doc);

    // Test undo
    let result = editor.undo();
    assert!(result.is_ok());

    // Verify the task item text was restored
    let doc = editor.document().borrow();
    match &doc.nodes[0] {
        Node::List { items, .. } => match &items[1].children[0] {
            Node::Paragraph { children } => match &children[0] {
                InlineNode::Text(text_node) => {
                    assert_eq!(text_node.text, "Task 2");
                }
                _ => panic!("Expected text node"),
            },
            _ => panic!("Expected paragraph node"),
        },
        _ => panic!("Expected list node"),
    }

    // Drop borrow before continuing
    drop(doc);

    // Test editing with invalid item index
    let result = editor.edit_task_item(0, 10, "Invalid");
    assert!(result.is_err());

    // Test editing with invalid node index
    let result = editor.edit_task_item(10, 0, "Invalid");
    assert!(result.is_err());

    // Test editing an item that has no paragraph node
    // Create a document with a task list item with no paragraph
    let mut document = Document::new();

    // Create an empty list item with no paragraph
    let mut items = vec![ListItem::new(vec![])];
    items[0].checked = Some(false);

    let list = Node::List {
        list_type: ListType::Task,
        items,
    };

    document.nodes.push(list);

    let mut editor = Editor::new(document);

    // Edit the item without paragraph
    let result = editor.edit_task_item(0, 0, "New paragraph");
    assert!(result.is_ok());

    // Verify the paragraph was created
    let doc = editor.document().borrow();
    match &doc.nodes[0] {
        Node::List { items, .. } => match &items[0].children[0] {
            Node::Paragraph { children } => match &children[0] {
                InlineNode::Text(text_node) => {
                    assert_eq!(text_node.text, "New paragraph");
                }
                _ => panic!("Expected text node"),
            },
            _ => panic!("Expected paragraph node"),
        },
        _ => panic!("Expected list node"),
    }
}
