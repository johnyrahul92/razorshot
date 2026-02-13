pub fn show_error(window: Option<&gtk4::Window>, title: &str, detail: &str) {
    log::error!("{title}: {detail}");
    let dialog = gtk4::AlertDialog::builder()
        .modal(true)
        .message(title)
        .detail(detail)
        .build();
    dialog.show(window);
}
