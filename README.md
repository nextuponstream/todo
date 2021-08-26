# todo
A utility CLI tool to create todo's with tags and deadlines. Todo's are saved to your specified folder (preferably a synchronized folder). You need to create a `.env` configuration file (following .env.example) for this tool to work.

## Build from source
```bash
git clone https://github.com/nextuponstream/todo.git
cd todo
cargo build --release
# Add binary to PATH
PATH=$PATH:</path/to/todo>/target/release/todo # edit here
sudo ln -s </path/to/todo>/target/release/todo /bin/todo
todo --version # has everything been set?
```

**Note:** before publishing to crates.io, I want to test it a fair bit myself to discover any missing feature to suit my needs.
