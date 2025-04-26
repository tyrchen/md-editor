use md_core::{Document, Editor, ListType, Node};

#[test]
fn test_remove_task_item_integration() {
    // Create a document with a task list
    let mut doc = Document::new();
    let task_list_idx =
        doc.add_task_list(vec![("Task 1", false), ("Task 2", true), ("Task 3", false)]);

    // Verify initial state
    match &doc.nodes[task_list_idx] {
        Node::List { list_type, items } => {
            assert_eq!(*list_type, ListType::Task);
            assert_eq!(items.len(), 3);
            assert_eq!(items[0].checked, Some(false));
            assert_eq!(items[1].checked, Some(true));
            assert_eq!(items[2].checked, Some(false));
        }
        _ => panic!("Expected a task list"),
    }

    // Create an editor
    let mut editor = Editor::new(doc);

    // Remove the middle task item
    let result = editor.remove_task_item(task_list_idx, 1);
    assert!(result.is_ok(), "Failed to remove task item: {:?}", result);

    // Verify the task was removed
    {
        let doc = editor.document().borrow();
        match &doc.nodes[task_list_idx] {
            Node::List { items, .. } => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].checked, Some(false)); // Task 1
                assert_eq!(items[1].checked, Some(false)); // Task 3
            }
            _ => panic!("Expected a task list"),
        }
    }

    // Test undo functionality
    let result = editor.undo();
    assert!(result.is_ok(), "Failed to undo: {:?}", result);

    // Verify the task was restored
    {
        let doc = editor.document().borrow();
        match &doc.nodes[task_list_idx] {
            Node::List { items, .. } => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0].checked, Some(false)); // Task 1
                assert_eq!(items[1].checked, Some(true)); // Task 2
                assert_eq!(items[2].checked, Some(false)); // Task 3
            }
            _ => panic!("Expected a task list"),
        }
    }

    // Test error handling for invalid indices
    let result = editor.remove_task_item(task_list_idx, 10);
    assert!(result.is_err(), "Expected error but command succeeded");

    let result = editor.remove_task_item(10, 0);
    assert!(result.is_err(), "Expected error but command succeeded");

    // Create a document with only one task
    let mut doc = Document::new();
    let task_list_idx = doc.add_task_list(vec![("Only task", true)]);
    let mut editor = Editor::new(doc);

    // Test trying to remove the only item
    let result = editor.remove_task_item(task_list_idx, 0);
    assert!(result.is_err(), "Expected error but command succeeded");
}
