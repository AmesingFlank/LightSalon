use salon_core::editor::Edit;
use salon_tests::test_utils::{enumerate_tests, ImageEditTest, TestContext};

#[test]
fn test_image_edit() {
    let mut test_context = TestContext::new();
    let tests = enumerate_tests();
    assert_eq!(tests.len(), 1);

    for test in tests {
        run_test(&mut test_context, test);
    }
}

fn run_test(test_context: &mut TestContext, test: ImageEditTest) {
    let session = &mut test_context.session;
    let original_image_identifier = session
        .library
        .add_single_item_from_path(test.original_image_path, None);
    let original_image = session
        .library
        .get_image_from_identifier(&original_image_identifier)
        .expect("failed to get original image");
    let expected_image_identifier = session
        .library
        .add_single_item_from_path(test.expected_image_path, None);
    let expected_image = session
        .library
        .get_image_from_identifier(&expected_image_identifier)
        .expect("failed to get expected image");
    session
        .editor
        .set_current_image(original_image_identifier.clone(), original_image.clone());

    let edit_json_str =
        std::fs::read_to_string(&test.edit_json_path).expect("failed to read edit json");
    let edit = serde_json::from_str::<Edit>(edit_json_str.as_str())
        .expect("failed to parse edit json str");

    session.editor.update_transient_edit(edit, false);
    session.editor.commit_transient_edit(false);
    let editted_image = session.editor.get_full_size_editted_image();

    test_context
        .image_comparer
        .assert_eq(editted_image, expected_image, 0.0001);
}
