# raingame-rs

![ScreenShot 2023-05-01 at 22 01 47](https://user-images.githubusercontent.com/37946887/235504907-e97281dd-446e-415a-81c0-0e45514a9fd1.gif)

raingame-rs는 1인 또는 2인으로 플레이 가능한 타이핑 게임입니다. 하늘에서 떨어지는 단어를 입력해서 단어 비로부터 살아남아보세요!

단어를 입력해서 스코어가 올라갈수록 단어는 점점 빠르게 떨어집니다.

2인에서 플레이하는 경우, 우측 상단의 ATTACK 단어를 입력하면 상대방에게 추가 단어 비를 내리게 할 수 있습니다!

## 설치 방법
- [release binary](https://github.com/KMUCS23-RUST-A/raingame-rs/releases)
- source code

### 클라이언트 실행 방법
- binary: `./client [--host <hostname> (default: 0.0.0.0)] [--port <port> (default: 22345)]`
- source: `cargo run --bin client -- [--host <hostname> (default: 0.0.0.0)] [--port <port> (default: 22345)]`

### 서버 실행 방법
- binary: `./server [--port <port> (default: 22345)]`
- source: `cargo run --bin client -- [--port <port> (default: 22345)]`

## Build
### 요구 사항
- Rust
- ncurses ^5.98
  - Ubuntu: `sudo apt install libncurses-dev`
  - CentOS/RHEL: `sudo dnf install ncurses-devel` or `sudo yum install ncurses-devel`
### build command
- `cargo build`

## 구조
<img src="https://user-images.githubusercontent.com/37946887/235584987-aa2282ec-58e0-4142-a2b0-1cc74ce64643.png" width="800">

## 라이선스

이 소프트웨어는 MIT 라이선스로 배포됩니다.
