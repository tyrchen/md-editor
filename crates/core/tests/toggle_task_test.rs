use md_core::{Document, Editor, ListType, Node};

#[test]
fn test_toggle_task_integration() {
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

    // Toggle the first task (unchecked -> checked)
    let result = editor.toggle_task(task_list_idx, 0);
    assert!(
        result.is_ok(),
        "Failed to execute toggle command: {:?}",
        result
    );

    // Verify the first task is now checked
    {
        let doc = editor.document().borrow();
        match &doc.nodes[task_list_idx] {
            Node::List { items, .. } => {
                assert_eq!(items[0].checked, Some(true));
                assert_eq!(items[1].checked, Some(true));
                assert_eq!(items[2].checked, Some(false));
            }
            _ => panic!("Expected a task list"),
        }
    }

    // Toggle the second task (checked -> unchecked)
    let result = editor.toggle_task(task_list_idx, 1);
    assert!(
        result.is_ok(),
        "Failed to execute toggle command: {:?}",
        result
    );

    // Verify the second task is now unchecked
    {
        let doc = editor.document().borrow();
        match &doc.nodes[task_list_idx] {
            Node::List { items, .. } => {
                assert_eq!(items[0].checked, Some(true));
                assert_eq!(items[1].checked, Some(false));
                assert_eq!(items[2].checked, Some(false));
            }
            _ => panic!("Expected a task list"),
        }
    }

    // Test undo functionality
    let result = editor.undo();
    assert!(
        result.is_ok(),
        "Failed to undo toggle command: {:?}",
        result
    );

    // Verify the second task is checked again
    {
        let doc = editor.document().borrow();
        match &doc.nodes[task_list_idx] {
            Node::List { items, .. } => {
                assert_eq!(items[0].checked, Some(true));
                assert_eq!(items[1].checked, Some(true));
                assert_eq!(items[2].checked, Some(false));
            }
            _ => panic!("Expected a task list"),
        }
    }

    // Test error handling for invalid indices
    let result = editor.toggle_task(task_list_idx, 10);
    assert!(result.is_err(), "Expected error but command succeeded");

    let result = editor.toggle_task(10, 0);
    assert!(result.is_err(), "Expected error but command succeeded");

    // Test with a non-task list by adding a paragraph
    {
        let mut doc = editor.document().borrow_mut();
        doc.add_paragraph_with_text("This is not a task list");
    }

    let result = editor.toggle_task(task_list_idx + 1, 0);
    assert!(result.is_err(), "Expected error but command succeeded");
}
