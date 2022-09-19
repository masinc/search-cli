# search-cli

This is a binary crate to search the web from CLI

# Usage

```
search [PROVIDER] WORD
```

`search` command with provider and word. The provider is optional. if not
specified, the first provider in the configuration file is used.

```bash
Usage:
    search searchword 
    search google searchword # provider is google
    search g searchword # provider is alias
```

# Configuration

The following command will output the configuration file path.

```
search config -p
```

Edit the output yaml path file with your favorite editor.

The following are the default settings.

```yaml
version: "v1.0"
providers:
  - name: google
    aliases:
      - g
    url: "https://google.com/search?q={{ word | urlencode }}"
  - name: bing
    url: "https://www.bing.com/search?q={{ word | urlencode }}"
  - name: duckduckgo
    aliases:
      - d
    url: "https://duckduckgo.com/?q={{ word | urlencode }}"
```

Each of these settings is described below.

## version

Currently fixed at `"v1.0"`.

## providers

An array of [`provider`](#provider)

## provider

| key                                                           | description                                                                   |
|---------------------------------------------------------------|-------------------------------------------------------------------------------|
| name                                                          | Provider Name. This is the name of the command execution.                     |
| aliases                                                       | An array of strings. A list of aliases for the command to execute.            |
| url                                                           | Search URL. `{{ word }}` inserts the contents of the `word` argument.         |
| browser                                                       | specify a browser name. See details for [provider.browser](#provider.browser) |

URLs are parsed using [`tera`](https://github.com/Keats/tera).                                                                             

### provider.browser

| key            | description                                          |
|----------------|------------------------------------------------------|
| default        | Use the OS default browser.                          |
| default_config | Use the default configuration browser.               |
| other          | Any browser name. e.g) `chrome`, `msedge`, `firefox` |

## default

example:

```yaml
version: "v1.0"
providers:
  ...
default:
  browser: chrome
```

`default` key is optional key.

| key     | description                                                      |
|---------|------------------------------------------------------------------|
| browser | specify a default browser. if not specified, OS default browser. |

# License

Mit or Apache-2.0
