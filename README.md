#ELP - AWS ELB Access Log Parser

ELP is a simple AWS ELB Access Log parser.  

See the [access log docs](http://docs.aws.amazon.com/ElasticLoadBalancing/latest/DeveloperGuide/access-log-collection.html)
for more information about them.

## Be kind and learn from others.

This project supports the [Rust Code of Conduct](https://www.rust-lang.org/conduct.html).

##How To

Add ELP as a dependency.  Add the following to your `Cargo.toml`:

```toml
[dependencies]
elp = "0.99.0"
```
Then reference it as an external crate in your code.

```rust
extern crate elp;
```

Here's a short program that uses an ELP utility method to get the paths of all
of the ELB access logs in a directory (recursively) and write the results of parsing
them to stdout.

```rust
extern crate elp;

fn main() {
  let mut filenames = Vec::new();
  match elp::file_list(log_location, &mut filenames) {
      Ok(num_files) => {
          elp::process_files(&filenames, &mut |parsing_result: ParsingResult| {
              println!("{:?}", parsing_result);
          });
          std::process::exit(0);
      },

      Err(e) => {
          println_stderr!("The following error occurred while trying to get the list of files. {}", e);
          std::process::exit(1);
      },
}
```
