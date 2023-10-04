// Initial versions:
//
// fn bar(value: &str) -> Option<&str> {
//     // ...
// }
//
// fn _foo<'a>(input: Vec<&'a str>) -> Vec<(usize, &'a str)> {
//     let mut output = Vec::new();
//     let mut input_index = 0;
//     let mut output_index = 0;
//     while input_index < input.len() {
//         let value = bar(input[input_index]);
//         if value.is_some() {
//             output.push((output_index, value.unwrap()));
//             output_index += 1;
//         }
//         input_index += 1;
//     }
//     output
// }

// Applied rework from top to bottom:

// * Overcomplicated logic with indexes, single variables current index is enough to manage such logic.
// * Usage unwrap - is a way to use unrecoverable errors in Rust.
// * Non-idiomatic way to handle Option. Better use `if let Some(value) = value {` or `map`, or `map_or`, instead
// * [Optional] Since the foo function uses Option<T>, there is nice to have minimal error handling.
// * Vec is a collection type in Rust, which allows us to iterate over its elements using iterator. A better version should be based on iterators.
// * Moving to iterators allows us to apply `filter-map-reduce(or -fold)` pattern in the implementation
//   Consequently, if the function foo depends on the bar function, the best candidate is filter_map.
// * The signature can be changed to `fn foo<T: AsRef<str>>(input: Vec<T>) -> Vec<(usize, T)>`,
//   which will allow to acceptance of either vector of string slices or a vector of strings or something that implements AsRef<str> trait.
// * Finally, end up with two versions of improved functions foo:

// V1:
// The bar function contract is unchanged
// The foo function filters the values using the bar's return value + is_some
mod v1 {
    fn bar(value: &str) -> Option<&str> {
        Some(value)
    }

    pub fn foo<T>(input: Vec<T>) -> Vec<(usize, T)>
    where
        T: AsRef<str>,
    {
        input
            .into_iter()
            .filter(|value| bar(value.as_ref()).is_some())
            .enumerate()
            .collect()
    }
}

// V2:
// The bar function contract is based on `T: AsRef<str>`,
// this change is required becase of filter_map's output iterator type
// would not match the return type of foo when calling the collect function:
//     `(usize, &str)` vs `(usize, T)`.
// The foo funciion filters the values using filter_map in a single line
mod v2 {
    fn bar<T: AsRef<str>>(value: T) -> Option<T> {
        Some(value)
    }

    pub fn foo<T>(input: Vec<T>) -> Vec<(usize, T)>
    where
        T: AsRef<str>,
    {
        input
            .into_iter()
            .filter_map(|val| bar(val))
            .enumerate()
            .collect()
    }
}

pub fn run() {
    let slice_input = vec!["string_0", "string_1", "string_2", ""];
    let string_input = vec![
        String::from("string_0"),
        String::from("string_1"),
        String::from("string_2"),
        String::from(""),
    ];

    // v1
    let slice_output = v1::foo(slice_input.clone());
    for i in slice_output {
        println!("{:?}", i);
    }

    let string_output = v1::foo(string_input.clone());
    for i in string_output {
        println!("{:?}", i);
    }

    // v2
    let slice_output = v2::foo(slice_input);
    for i in slice_output {
        println!("{:?}", i);
    }

    let string_output = v2::foo(string_input);
    for i in string_output {
        println!("{:?}", i);
    }
}
