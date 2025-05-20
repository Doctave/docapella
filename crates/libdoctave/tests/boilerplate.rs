use libdoctave::Project;
use libdoctave::{InputContent, InputFile};

#[test]
/// Ensure we can build the boilerplate project
fn sanity() {
    let files = Project::boilerplate_file_list();
    let files = files
        .into_iter()
        .map(|(path, content)| {
            if path.starts_with("_assets") {
                InputFile {
                    path,
                    content: InputContent::Binary("".to_string()),
                }
            } else {
                InputFile {
                    path,
                    content: InputContent::Text(
                        std::str::from_utf8(content.as_slice()).unwrap().to_owned(),
                    ),
                }
            }
        })
        .collect();

    let project = Project::from_file_list(files);

    assert!(
        project.is_ok(),
        "Failed to create project from boilerplate: {:?}",
        project
    );

    let verification = project.unwrap().verify(None, None);
    assert!(
        verification.is_ok(),
        "Failed to verify boilerplate project: {:?}",
        verification
    );
}
