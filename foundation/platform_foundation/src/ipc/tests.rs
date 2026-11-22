use std::io::{self, Read, Write};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use super::Endpoint;

/// 試験の口のまとまりの名前（利用者の実際の口と分ける）
const GROUP: &str = "RedRingTest";

/// 試験ごとの口
fn test_endpoint() -> Endpoint {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    Endpoint::for_current_user(
        GROUP,
        &format!(
            "test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ),
    )
    .unwrap()
}

#[test]
fn bytes_go_both_ways_through_the_endpoint() {
    let endpoint = test_endpoint();
    let mut listener = endpoint.listen(4).unwrap();
    let server = std::thread::spawn(move || {
        let mut connection = listener.accept().unwrap();
        let mut request = [0u8; 4];
        connection.read_exact(&mut request).unwrap();
        connection.write_all(&request.map(|byte| byte + 1)).unwrap();
        connection.flush().unwrap();
    });
    let mut connection = endpoint.connect(Duration::from_secs(5)).unwrap();
    connection.write_all(&[1, 2, 3, 4]).unwrap();
    connection.flush().unwrap();
    let mut answer = [0u8; 4];
    connection.read_exact(&mut answer).unwrap();
    assert_eq!(answer, [2, 3, 4, 5]);
    server.join().unwrap();
}

#[test]
fn connecting_to_a_closed_endpoint_fails_without_waiting() {
    let endpoint = test_endpoint();
    assert_eq!(
        endpoint.connect(Duration::ZERO).unwrap_err().kind(),
        io::ErrorKind::NotFound
    );
    // 閉じた口は、開き直せる
    drop(endpoint.listen(4).unwrap());
    let _listener = endpoint.listen(4).unwrap();
    // 開いている口は、2 つ目を開かない（UNIX では、開いているかを確かめるために一度つなぐため、接続を受ける試験と
    // 分ける）
    assert_eq!(
        endpoint.listen(4).unwrap_err().kind(),
        io::ErrorKind::AddrInUse
    );
}

#[test]
fn the_endpoint_stays_open_while_its_only_connection_comes_and_goes() {
    let endpoint = test_endpoint();
    let mut listener = endpoint.listen(1).unwrap();
    let server = std::thread::spawn(move || {
        let connection = listener.accept().unwrap();
        (listener, connection)
    });
    let client = endpoint.connect(Duration::from_secs(5)).unwrap();
    let (_listener, connection) = server.join().unwrap();
    // 1 つだけつなげる口でも、接続を受けて閉じた後に、ほかが同じ口を開けない
    drop(connection);
    drop(client);
    assert_eq!(
        endpoint.listen(1).unwrap_err().kind(),
        io::ErrorKind::AddrInUse
    );
}

#[test]
fn a_connection_over_the_limit_is_served_after_a_connection_closes() {
    let endpoint = test_endpoint();
    let mut listener = endpoint.listen(1).unwrap();
    let (accepted, first) = std::sync::mpsc::channel();
    let server = std::thread::spawn(move || {
        accepted.send(listener.accept().unwrap()).unwrap();
        // 受けた接続が閉じた後の接続を受ける
        let mut connection = listener.accept().unwrap();
        connection.write_all(&[7]).unwrap();
        connection.flush().unwrap();
    });
    let _first_client = endpoint.connect(Duration::from_secs(5)).unwrap();
    let first = first.recv().unwrap();

    // 上限を超えた接続は、Windows では空くまで待たされ、ほかの OS では閉じられる（待ち続けない）
    let mut over = endpoint.connect(Duration::from_secs(5)).unwrap();
    over.set_io_timeout(Some(Duration::from_secs(5))).unwrap();
    let mut byte = [0u8; 1];
    if cfg!(windows) {
        drop(first);
        over.read_exact(&mut byte).unwrap();
    } else {
        match over.read(&mut byte) {
            Ok(read) => assert_eq!(read, 0),
            Err(error) => assert_ne!(error.kind(), io::ErrorKind::TimedOut, "{error:?}"),
        }
        drop(first);
        let mut next = endpoint.connect(Duration::from_secs(5)).unwrap();
        next.set_io_timeout(Some(Duration::from_secs(5))).unwrap();
        next.read_exact(&mut byte).unwrap();
    }
    assert_eq!(byte, [7]);
    server.join().unwrap();
}

#[test]
fn a_silent_other_side_times_out() {
    let endpoint = test_endpoint();
    let mut listener = endpoint.listen(4).unwrap();
    // 接続を受けても、何も書かない受け手
    let (release, released) = std::sync::mpsc::channel::<()>();
    let server = std::thread::spawn(move || {
        let _connection = listener.accept().unwrap();
        let _ = released.recv();
    });
    let mut connection = endpoint.connect(Duration::from_secs(5)).unwrap();
    assert!(connection.set_io_timeout(Some(Duration::ZERO)).is_err());
    connection
        .set_io_timeout(Some(Duration::from_millis(200)))
        .unwrap();
    let started = std::time::Instant::now();
    let error = connection.read(&mut [0u8; 1]).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::TimedOut, "{error:?}");
    assert!(started.elapsed() < Duration::from_secs(5));
    release.send(()).unwrap();
    server.join().unwrap();
}

#[test]
fn names_that_could_leave_the_folder_are_not_used() {
    for (group, name) in [
        ("", "name"),
        ("group", ""),
        ("bad/group", "name"),
        ("group", "bad\\name"),
        ("..", "name"),
        ("group", ".hidden"),
        // まとまりに `.` を使うと、Windows で別のまとまりと名前の組と同じパイプの名前になる
        ("RedRing.JobManager", "x"),
    ] {
        assert_eq!(
            Endpoint::for_current_user(group, name).unwrap_err().kind(),
            io::ErrorKind::InvalidInput,
            "{group:?} {name:?}"
        );
    }
    assert!(Endpoint::for_current_user(&"a".repeat(65), "name").is_err());
    assert!(Endpoint::for_current_user("RedRing", "JobManager.test-1").is_ok());
}

#[test]
fn the_number_of_connections_must_be_in_range() {
    let endpoint = test_endpoint();
    for count in [0, super::MAX_CONNECTIONS_LIMIT + 1] {
        assert_eq!(
            endpoint.listen(count).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
    }
}
