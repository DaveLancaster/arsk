# Arsk

Prompt for user input in a composable way.

## Installation

Add this line to your application's Cargo.toml:
```
[dependencies]
arsk = { git = "https://github.com/DaveLancaster/arsk" }
```

## Usage

You can prompt users for input like so:
```
let colour = Arsk::Colour::Red;
let resp = Arsk::input("What shall we do today?").prompt(&':').fg_colour(colour).ask().unwrap();
```

## Contributing

Bug reports and pull requests are welcome on GitHub at https://github.com/DaveLancaster/arsk.

## License

The crate is available as open source under the terms of the [MIT License](http://opensource.org/licenses/MIT).

