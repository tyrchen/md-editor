use md_core::{Document, Editor, ListItem, ListType, Node};

#[test]
fn test_move_task_item_integration() {
    // Create a document with a task list containing three items
    let mut document = Document::new();

    let items = vec![
        ListItem::task("Task 1", false),
        ListItem::task("Task 2", true),
        ListItem::task("Task 3", false),
    ];

    let list = Node::List {
        list_type: ListType::Task,
        items,
    };

    document.nodes.push(list);

    let mut editor = Editor::new(document);

    // Verify initial state
    let doc = editor.document().borrow();
    assert_eq!(doc.nodes.len(), 1);

    match &doc.nodes[0] {
        Node::List { items, .. } => {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0].as_text(), Some("Task 1"));
            assert_eq!(items[1].as_text(), Some("Task 2"));
            assert_eq!(items[2].as_text(), Some("Task 3"));
        }
        _ => panic!("Expected list node"),
    }

    // Drop borrow before continuing
    drop(doc);

    // Move Task 1 to the end of the list
    let result = editor.move_task_item(0, 0, 2);
    assert!(result.is_ok());

    // Verify the task was moved
    let doc = editor.document().borrow();
    match &doc.nodes[0] {
        Node::List { items, .. } => {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0].as_text(), Some("Task 2"));
            assert_eq!(items[1].as_text(), Some("Task 3"));
            assert_eq!(items[2].as_text(), Some("Task 1"));

            // Check that Task 2 is still checked
            assert_eq!(items[0].checked, Some(true));
        }
        _ => panic!("Expected list node"),
    }

    // Drop borrow before continuing
    drop(doc);

    // Test undo
    let result = editor.undo();
    assert!(result.is_ok());

    // Verify the task order was restored
    let doc = editor.document().borrow();
    match &doc.nodes[0] {
        Node::List { items, .. } => {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0].as_text(), Some("Task 1"));
            assert_eq!(items[1].as_text(), Some("Task 2"));
            assert_eq!(items[2].as_text(), Some("Task 3"));
        }
        _ => panic!("Expected list node"),
    }

    // Drop borrow before continuing
    drop(doc);

    // Test invalid indices
    let result = editor.move_task_item(0, 5, 1);
    assert!(result.is_err());

    let result = editor.move_task_item(0, 0, 5);
    assert!(result.is_err());

    let result = editor.move_task_item(5, 0, 1);
    assert!(result.is_err());
}
