use md_core::{Document, Editor, ListType, Node};

#[test]
fn test_add_task_item_integration() {
    // Create a document with a task list
    let mut doc = Document::new();
    let task_list_idx = doc.add_task_list(vec![("Task 1", false), ("Task 2", true)]);

    // Verify initial state
    match &doc.nodes[task_list_idx] {
        Node::List { list_type, items } => {
            assert_eq!(*list_type, ListType::Task);
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].checked, Some(false));
            assert_eq!(items[1].checked, Some(true));
        }
        _ => panic!("Expected a task list"),
    }

    // Create an editor
    let mut editor = Editor::new(doc);

    // Add a new task item at the end
    let result = editor.add_task_item(task_list_idx, 2, "Task 3", false);
    assert!(result.is_ok(), "Failed to add task item: {:?}", result);

    // Verify the task was added at the end
    {
        let doc = editor.document().borrow();
        match &doc.nodes[task_list_idx] {
            Node::List { items, .. } => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].checked, Some(false));
                assert_eq!(items[1].checked, Some(true));
                assert_eq!(items[2].checked, Some(false));
            }
            _ => panic!("Expected a task list"),
        }
    }

    // Add a new task item at the beginning
    let result = editor.add_task_item(task_list_idx, 0, "Task 0", true);
    assert!(result.is_ok(), "Failed to add task item: {:?}", result);

    // Verify the task was added at the beginning
    {
        let doc = editor.document().borrow();
        match &doc.nodes[task_list_idx] {
            Node::List { items, .. } => {
                assert_eq!(items.len(), 4);
                assert_eq!(items[0].checked, Some(true));
                assert_eq!(items[1].checked, Some(false));
                assert_eq!(items[2].checked, Some(true));
                assert_eq!(items[3].checked, Some(false));
            }
            _ => panic!("Expected a task list"),
        }
    }

    // Test undo functionality
    let result = editor.undo();
    assert!(result.is_ok(), "Failed to undo: {:?}", result);

    // Verify the first task was removed
    {
        let doc = editor.document().borrow();
        match &doc.nodes[task_list_idx] {
            Node::List { items, .. } => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].checked, Some(false));
                assert_eq!(items[1].checked, Some(true));
                assert_eq!(items[2].checked, Some(false));
            }
            _ => panic!("Expected a task list"),
        }
    }

    // Undo again
    let result = editor.undo();
    assert!(result.is_ok(), "Failed to undo: {:?}", result);

    // Verify we're back to the original state
    {
        let doc = editor.document().borrow();
        match &doc.nodes[task_list_idx] {
            Node::List { items, .. } => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].checked, Some(false));
                assert_eq!(items[1].checked, Some(true));
            }
            _ => panic!("Expected a task list"),
        }
    }

    // Test error handling for invalid indices
    let result = editor.add_task_item(task_list_idx, 10, "Invalid position", false);
    assert!(result.is_err(), "Expected error but command succeeded");

    let result = editor.add_task_item(10, 0, "Invalid node", false);
    assert!(result.is_err(), "Expected error but command succeeded");

    // Test with a non-task list by adding a paragraph
    {
        let mut doc = editor.document().borrow_mut();
        doc.add_paragraph_with_text("This is not a task list");
    }

    let result = editor.add_task_item(task_list_idx + 1, 0, "Invalid list type", false);
    assert!(result.is_err(), "Expected error but command succeeded");
}
