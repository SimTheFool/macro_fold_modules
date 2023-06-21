mod folder;
use folder::Folder;

#[test]
fn test() {
    let folder = Folder::new(1, 2);
    let file_a = folder.file_a;
    let file_b = folder.file_b;

    assert!(file_a.x == 4);
    assert!(file_a.y == 5);
    assert!(file_b.x == 1);
    assert!(file_b.y == 2);
}
