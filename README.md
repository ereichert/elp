#ELP - AWS ELB Access Log Parser

ELP is a simple AWS ELB Access Log parser.  

See the [access log docs](http://docs.aws.amazon.com/ElasticLoadBalancing/latest/DeveloperGuide/access-log-collection.html)
for more information about them.

## Be kind and learn from others.

This project supports the [Rust Code of Conduct](https://www.rust-lang.org/conduct.html).

## [Documentation](http://ereichert.github.io/elp/elp/index.html)

##How To

There is a full example of ELP's use in a production environment [here](https://github.com/trafficland/counter).

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

  // Get a list of files from a directory specified by the user.
  match elp::file_list(log_location, &mut filenames) {
      // If walking the directory succeeds
      Ok(_) => {
          // Attempt to parse each record in each file passing the result to
          // a user defined result handler.
          elp::process_files(&filenames, &mut |parsing_result: ParsingResult| {
              println!("{:?}", parsing_result);
          });

          std::process::exit(0);
      },

      Err(e) => {
          println!("The following error occurred while trying to get the list of files. {}", e);
          std::process::exit(1);
      },
}
```

Most of this is pretty standard Rust code.  The only ELP specific code of note
is the handler.

An attempt is made to parse every record.  The results of the attempt to parse
each record is passed to a user defined handler having the following function
signature.

```rust
FnMut(ParsingResult) -> ()
```

It's up to the user to check for errors.

Why a handler and not return a Vec or some other collection of ELBRecord?

If you run an ELB with heavy traffic you can easily produce millions of records
per day.  Storing the records in memory and returning them is not viable for
high load ELB.  By providing a handler to which each record will be passed the
user can decide how to handle each record whether it be storing it in memory or
writing them to disk.
