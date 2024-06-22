# Atomizer

A terminal Atom feed reader

## Usage

```console
atomizer --help
Usage: atomizer <COMMAND>

Commands:
  read        Read your feeds
  categories  Query the categories in your feeds
  update      Update the feeds you've added
  setup       Setup the data directories
  add         Add a feed
  remove      Remove a feed
  config      Interact with Configuration
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### `read`

```console
atomizer read --help
Read your feeds

Usage: atomizer read [OPTIONS]

Options:
  -n, --no-update            Skip re-fetching the feed
  -c, --category <CATEGORY>  Filter entries by a category
  -a, --all                  Include previously read entries
  -h, --help                 Print help
```

### `categories`

```console
atomizer categories --help
Query the categories in your feeds

Usage: atomizer categories

Options:
  -h, --help  Print help
```

### `update`

```console
atomizer update --help
Update the feeds you've added

Usage: atomizer update

Options:
  -h, --help  Print help
```

### `setup`

```console
atomizer setup --help
Setup the data directories

Usage: atomizer setup [OPTIONS]

Options:
  -f, --force  
  -h, --help   Print help
```

### `add`

```console
atomizer add --help
Add a feed

Usage: atomizer add <URL>

Arguments:
  <URL>  

Options:
  -h, --help  Print help
```

### `remove`

```console
atomizer remove --help
Remove a feed

Usage: atomizer remove <ID_OR_NAME>

Arguments:
  <ID_OR_NAME>  

Options:
  -h, --help  Print help
```

### `config`

```console
atomizer config --help
Interact with Configuration

Usage: atomizer config [OPTIONS] [KEY] [VALUE]

Arguments:
  [KEY]    If a value is provided, the key to assign the value to if no value is provided print the configuration key's value
  [VALUE]  The value to assign to the key

Options:
  -d, --delete  The provided key will be removed
  -h, --help    Print help
```
