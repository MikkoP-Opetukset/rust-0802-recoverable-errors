#![allow(dead_code, unused_variables, unused_mut)]

use std::error::Error;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;

/// Entry point for Chapter 9.2: Recoverable Errors with `Result`
///
/// Most errors are not serious enough to stop the whole program. When a
/// function fails for a reason the caller can understand, the failure is a
/// value: `Result<T, E>` with an `Ok` or an `Err`. This file covers matching on
/// `Result`, the `unwrap` and `expect` shortcuts, propagating errors with `?`,
/// and using `?` on `Option`.
///
/// The examples use one save file in this demo directory (`save_slot_1.txt`,
/// which is in `.gitignore`). Each section removes it first so the output is
/// the same on every run. Panicking arms are present but not executed; delete
/// the file, comment out the example functions prior to the one you want to
/// inspect, and run again to see them.
fn main() {
    result_is_an_enum();
    matching_on_a_result();
    matching_on_different_errors();
    unwrap_and_expect();
    propagating_errors_with_match();
    propagating_errors_with_question_mark();
    using_question_mark_on_option();
    where_the_question_mark_can_be_used();
}

/// The save file used by the examples.
const SAVE_FILE: &str = "save_slot_1.txt";

/// Remove the save file so every section starts from the same state.
fn fresh_save() {
    let _ = fs::remove_file(SAVE_FILE);
}

/// Read the character name stored in the save file.
fn read_name(save: &mut File) -> String {
    let mut name = String::new();
    save.read_to_string(&mut name)
        .expect("the file was just created, so reading should work");
    name
}

/// # `Result<T, E>` Is an Enum
/// - `enum Result<T, E> { Ok(T), Err(E) }`: `Ok` holds the success value and
///   `Err` holds information about what went wrong.
/// - `T` and `E` are generic type parameters, so the success and error types
///   can be anything. For `File::open` the implementation fills them in as
///   `std::fs::File` and `std::io::Error`.
/// - Like `Option`, `Result` comes from the prelude, so arms are written as
///   `Ok(file)` and `Err(error)` without a `Result::` prefix.
fn result_is_an_enum() {
    println!("\n{:=>80}", "");
    println!("result_is_an_enum()\n");

    fresh_save();

    // The file does not exist yet, so the value is an Err carrying the reason.
    match File::open(SAVE_FILE) {
        Ok(_) => println!("Ok branch: a handle we can read from."),
        Err(error) => println!("Err branch: {error}"),
    }

    // The same call now succeeds, because the file exists.
    File::create(SAVE_FILE).expect("the directory should be writable");
    match File::open(SAVE_FILE) {
        Ok(_) => println!("Ok branch again: the same call can now succeed."),
        Err(error) => println!("Err branch: {error}"),
    }
}

/// # Matching on a Result
/// - `match` is the basic tool for handling the two variants.
/// - The `Ok` arm returns the inner value, here the file handle, which we then
///   use for reading or writing.
/// - The `Err` arm is a choice, not a requirement. `panic!` is only right when
///   there is nothing we can do about the failure.
fn matching_on_a_result() {
    println!("\n{:=>80}", "");
    println!("matching_on_a_result()\n");

    fresh_save();
    fs::write(SAVE_FILE, "Aria").expect("the directory should be writable");

    let mut save = match File::open(SAVE_FILE) {
        Ok(file) => file,
        // This arm is written but not executed, because the file exists.
        // Delete the file and run again to see it panic:
        //
        //   thread 'main' panicked at src/main.rs:LINE:COL:
        //   Problem opening the file: Os { code: 2, kind: NotFound, message: "No such file or directory" }
        //
        Err(error) => panic!("Problem opening the file: {error:?}"),
    };

    println!("After the match, `save` is the handle from the Ok variant: {save:?}");
    println!("Reading the file gives: {}", read_name(&mut save));
}

/// # Matching on Different Errors
/// - Matching only on `Ok` and `Err` treats every failure the same way.
/// - `io::Error` has a `kind()` method that returns an `io::ErrorKind`, whose
///   variants name the reason for the failure.
/// - A missing file is `ErrorKind::NotFound`, so that case can be fixed by
///   creating the file while every other reason still panics.
/// - `File::create` can fail too, so the inner match needs its own arm.
fn matching_on_different_errors() {
    println!("\n{:=>80}", "");
    println!("matching_on_different_errors()\n");

    fresh_save();

    let mut save = match File::open(SAVE_FILE) {
        Ok(file) => file,
        Err(error) => match error.kind() {
            ErrorKind::NotFound => match File::create(SAVE_FILE) {
                Ok(mut created) => {
                    println!("NotFound is a case we can handle: the file is created.");
                    // The new handle is write-only, so the name is written here.
                    created
                        .write_all(b"Aria")
                        .expect("the new file should be writable");
                    created
                }
                // Not executed here. It would run if creating failed, for
                // example in a directory we cannot write to.
                Err(e) => panic!("Problem creating the file: {e:?}"),
            },
            // Not executed here. Any other kind of failure ends the program.
            _ => panic!("Problem opening the file: {error:?}"),
        },
    };

    // Both arms produce a handle, which is what the nested match is for.
    println!("Either way we end up with a handle: {save:?}");

    // The handle from File::create is write-only, so reading it fails.
    // Uncomment to see:
    //
    //   thread 'main' panicked at src/main.rs:LINE:COL:
    //   the file was just created, so reading should work: Os { code: 9, kind: Uncategorized, message: "Bad file descriptor" }
    //
    // println!("Reading through the created handle: {}", read_name(&mut save));

    // Reading needs a handle opened for reading, so the file is opened again.
    println!(
        "Reading the file through a fresh open gives: {}",
        fs::read_to_string(SAVE_FILE).expect("the file was just created")
    );
}

/// # Shortcuts for Panic on Error
/// - `unwrap` is the `match` from the previous section as one call: it returns
///   the inner `Ok` value or calls `panic!` for us on an `Err`.
/// - `expect` does the same but lets us choose the message, which conveys
///   intent and makes a panic easier to trace.
/// - In production-quality code most Rustaceans use `expect` rather than
///   `unwrap`, because the message says what the code assumed.
fn unwrap_and_expect() {
    println!("\n{:=>80}", "");
    println!("unwrap_and_expect()\n");

    fresh_save();
    fs::write(SAVE_FILE, "Aria").expect("the directory should be writable");

    // Both calls return the handle because the file exists.
    let mut save = File::open(SAVE_FILE).unwrap();
    println!("unwrap on an Ok value returns the handle.");

    let mut save = File::open(SAVE_FILE).expect("the save file should be in this demo");
    println!("expect returns the same handle, with our own message ready.");

    // Delete the file and run again to see the messages those calls produce:
    //
    //   thread 'main' panicked at src/main.rs:LINE:COL:
    //   called `Result::unwrap()` on an `Err` value: Os { code: 2, kind: NotFound, message: "No such file or directory" }
    //
    //   thread 'main' panicked at src/main.rs:LINE:COL:
    //   the save file should be in this demo: Os { code: 2, kind: NotFound, message: "No such file or directory" }
    //
    // The second one is easier to read, which is the point of `expect`.
    println!("Reading the file gives: {}", read_name(&mut save));
}

/// # Propagating Errors with `match`
/// - Instead of handling a failure inside the function, a function can return
///   the error to its caller. This is *propagating*, and it gives control to
///   the caller, which knows what the value was meant for.
/// - The return type `Result<String, io::Error>` means: success gives a
///   `String`, failure gives an `io::Error`. That type was chosen because both
///   operations in the body return `io::Error`.
/// - In the `Err` arm we use `return` to leave the function early instead of
///   panicking.
fn propagating_errors_with_match() {
    println!("\n{:=>80}", "");
    println!("propagating_errors_with_match()\n");

    fresh_save();
    fs::write(SAVE_FILE, "Aria").expect("the directory should be writable");

    match read_save_name_with_match(SAVE_FILE) {
        Ok(name) => println!("The caller receives Ok with the name: {name}"),
        Err(error) => println!("The caller receives Err with: {error}"),
    }
}

/// Read the character name from the save file, returning failures to the caller.
fn read_save_name_with_match(path: &str) -> Result<String, io::Error> {
    let save_result = File::open(path);

    let mut save_file = match save_result {
        Ok(file) => file,
        // The early return is the propagation: the error goes back to the
        // caller instead of ending the program here.
        Err(e) => return Err(e),
    };

    let mut name = String::new();

    // read_to_string returns a Result as well, even when opening succeeded.
    match save_file.read_to_string(&mut name) {
        Ok(_) => Ok(name),
        Err(e) => Err(e),
    }
}

/// # Propagating Errors with `?`
/// - `?` on a `Result` means: if it is `Ok`, use the inner value and continue;
///   if it is `Err`, return it from the whole function, as `return` would.
/// - The same behaviour as the `match` version, without the boilerplate.
/// - `?` also calls `from` from the `From` trait, so an error of one type can
///   become the error type named in the function's return type.
fn propagating_errors_with_question_mark() {
    println!("\n{:=>80}", "");
    println!("propagating_errors_with_question_mark()\n");

    fresh_save();
    fs::write(SAVE_FILE, "Aria").expect("the directory should be writable");

    match read_save_name_with_question_mark(SAVE_FILE) {
        Ok(name) => println!("Same result as the match version: {name}"),
        Err(error) => println!("Propagated error: {error}"),
    }

    match read_save_name_chained(SAVE_FILE) {
        Ok(name) => println!("Chained after `?`, same behaviour: {name}"),
        Err(error) => println!("Propagated error: {error}"),
    }

    // The `?` here converts io::Error into our own error type through `From`.
    match read_save_name_as_our_error(SAVE_FILE) {
        Ok(name) => println!("Converted to our own error type, name still {name}"),
        Err(error) => println!("Our error type: {error}"),
    }
}

/// The same function as above, written with `?`.
fn read_save_name_with_question_mark(path: &str) -> Result<String, io::Error> {
    let mut save_file = File::open(path)?;
    let mut name = String::new();
    save_file.read_to_string(&mut name)?;
    Ok(name)
}

/// The same function with the calls chained after `?`.
fn read_save_name_chained(path: &str) -> Result<String, io::Error> {
    let mut name = String::new();
    File::open(path)?.read_to_string(&mut name)?;
    Ok(name)
}

/// Reading a file into a string is common enough that the standard library has
/// a function for the whole operation, and its error type is already `io::Error`.
fn read_save_name_whole_file(path: &str) -> Result<String, io::Error> {
    fs::read_to_string(path)
}

/// An error type for this demo's save-file code.
#[derive(Debug)]
struct SaveError {
    message: String,
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for SaveError {}

impl From<io::Error> for SaveError {
    fn from(error: io::Error) -> Self {
        SaveError {
            message: format!("save file problem: {error}"),
        }
    }
}

/// `?` in this function calls `from` to turn an `io::Error` into a `SaveError`.
fn read_save_name_as_our_error(path: &str) -> Result<String, SaveError> {
    let mut name = String::new();
    File::open(path)?.read_to_string(&mut name)?;
    Ok(name)
}

/// # Using `?` on an `Option`
/// - `?` works on `Option` in a function that returns `Option`: a `None` is
///   returned early, a `Some` gives the inner value and the function continues.
/// - `?` does not convert between `Result` and `Option`. Use `ok()` on a
///   `Result` or `ok_or()` on an `Option` when a conversion is needed.
fn using_question_mark_on_option() {
    println!("\n{:=>80}", "");
    println!("using_question_mark_on_option()\n");

    println!(
        "last char of the first line: {:?}",
        last_char_of_first_line("Aria the knight\nHow are you today?")
    );
    println!("empty text: {:?}", last_char_of_first_line(""));
    println!("blank first line: {:?}", last_char_of_first_line("\nhi"));

    // The conversion between the two types is explicit, never automatic.
    let found = last_char_of_first_line("Aria");
    println!(
        "Result from an Option: {:?}",
        found.ok_or("no character found")
    );
}

/// The last character of the first line, which may not exist.
fn last_char_of_first_line(text: &str) -> Option<char> {
    text.lines().next()?.chars().last()
}

/// # Where the `?` Operator Can Be Used
/// - `?` can only be used in a function whose return type is compatible with
///   the value it is used on, because `?` is an early return out of the
///   function.
/// - `main` can return `Result<(), Box<dyn Error>>`, read as "any kind of
///   error". Such a program exits with 0 when `main` returns `Ok(())` and with
///   a nonzero value when it returns an `Err`, the same convention C programs
///   follow.
fn where_the_question_mark_can_be_used() {
    println!("\n{:=>80}", "");
    println!("where_the_question_mark_can_be_used()\n");

    // This function returns (), so `?` cannot be used in it. Uncomment the line
    // to see the compile error:
    //
    //   error[E0277]: the `?` operator can only be used in a function that
    //   returns `Result` or `Option` (or another type that implements
    //   `FromResidual`)
    //
    // let save = File::open(SAVE_FILE)?;

    // Changing the return type of the function is what fixes it:
    match read_save_name_whole_file(SAVE_FILE) {
        Ok(name) => println!("Reading through a Result-returning function: {name}"),
        Err(error) => println!("The caller decides what to do with: {error}"),
    }

    println!("The `?` cannot mix the two types: a Result-returning function");
    println!("cannot use `?` on an Option, and the other way round.");
}
