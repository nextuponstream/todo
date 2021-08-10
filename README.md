# todo
A utility CLI tool to create todo's with tags and deadlines. Todo's are saved to your specified folder (preferably a synchronized folder). You need to create a `.env` configuration file (following .env.example) for this tool to work.

## Install
```bash
git clone ???
cd todo
# Configure .env file following .env.example
cargo build
# Add binary to PATH
PATH=$PATH:/path/to/todo/target/debug
todo --version # has everything been set?
```
