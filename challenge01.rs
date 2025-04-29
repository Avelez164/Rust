use base64::{engine::general_purpose, Engine as _};
use hex;

fn main() {
    let input_string = "49276d206b696c6c696e6720796f757220627261\
                        696e206c696b65206120706f69736f6e6f7573206\
                        d757368726f6f6d";

    let bytes = hex::decode(input_string).expect("Failed to decode hex");

    let b64_string = general_purpose::STANDARD.encode(&bytes);

    println!("{}", b64_string);
}

// hex string: 49 27 6d 20 6b 69 6c 6c 69 6e 67 20 79 6f 75 72 20 62 72 61 69 6e 20 6c 69 6b 65 20 61 20 70 6f 69 73 6f 6e 6f 75 73 20 6d 75 73 68 72 6f 6f 6d
// Hex -> Decimal -> ASCII
// 49  -> 73      -> I
// 27  -> 39      -> '
// 6d  -> 109     -> m
// 20  -> 32      -> (space)
// 6b  -> 107     -> k
// 69  -> 105     -> i
// 6c  -> 108     -> l
// 6c  -> 108     -> l
// 69  -> 105     -> i
// 6e  -> 110     -> n
// 67  -> 103     -> g
// 20  -> 32      -> (space)
// 79  -> 121     -> y
// 6f  -> 111     -> o
// 75  -> 117     -> u
// 72  -> 114     -> r
// 20  -> 32      -> (space)
// 62  -> 98      -> b
// 72  -> 114     -> r
// 61  -> 97      -> a
// 69  -> 105     -> i
// 6e  -> 110     -> n
// 20  -> 32      -> (space)
// 6c  -> 108     -> l
// 69  -> 105     -> i
// 6b  -> 107     -> k
// 65  -> 101     -> e
// 20  -> 32      -> (space)
// 61  -> 97      -> a
// 20  -> 32      -> (space)
// 70  -> 112     -> p
// 6f  -> 111     -> o
// 69  -> 105     -> i
// 73  -> 115     -> s
// 6f  -> 111     -> o
// 6e  -> 110     -> n
// 6f  -> 111     -> o
// 75  -> 117     -> u
// 73  -> 115     -> s
// 20  -> 32      -> (space)
// 6d  -> 109     -> m
// 75  -> 117     -> u
// 73  -> 115     -> s
// 68  -> 104     -> h
// 72  -> 114     -> r
// 6f  -> 111     -> o
// 6f  -> 111     -> o
// 6d  -> 109     -> m
