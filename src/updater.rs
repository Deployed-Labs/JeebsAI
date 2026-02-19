pub async fn github_update_watcher() {
    // Minimal placeholder watcher — keeps task alive but does nothing for now.
    loop {
        // In future: poll GitHub for releases and notify `evolution` subsystem
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
