# wathcing_log_parser

## Introduction

This is a toy project for parsing watching logs using named regex.

It deploys multi-threading to speed up the parsing process and uses caching to avoid re-parsing the same logs.

## How to use

### General

```bash
> watching_record --help
Usage: watching_record [OPTIONS] --filename <FILENAME>

Options:
  -c, --config-path <CONFIG_PATH>  If not set, we will use your system's config path
  -f, --filename <FILENAME>
  -l, --log-level <LOG_LEVEL>      If not set, we will use warning leve. The options are: error, warn, info, debug. [default: warn] [possible values: error, warn
, info, debug]
  -m, --mode <MODE>                We have three mode right now,
                                        unfinished(default): list all unifhished watching
                                        query: list all matching watching with give query name
                                        all: list all watching.
                                    [default: un-finished] [possible values: un-finished, query, all]
  -q, --query-name <QUERY_NAME>
  -h, --help                       Print help
  -V, --version                    Print version
```

### Config file

Users can specify a config file path. If there is none, this program will try to find the config file in the system's config path.

:warning: The config file must exist, otherwise this program will exit with error.

The config file should contain sections below:

```yaml
reg_pattern_list:
  - '(?<name>.+)第(?<season>[0-9一二三四五六七八九十零百千]+)季\sSP\s(?<time_at_episode>\d{1,2}:\d{1,2}.*)\s(?<logged_time>\d{4}-\d{2}-\d{2}\s.*)$'
  - '(?<name>.+)第(?<season>[0-9一二三四五六七八九十零百千]+)季\s第(?<episode>[0-9一二三四五六七八九十零百千]+)集\s(?<time_at_episode>\d{1,2}:\d{1,2}.*)\s(?<logged_time>\d{4}-\d{2}-\d{2}\s.*)$'
  - '(?<name>.+)第(?<season>[0-9一二三四五六七八九十零百千]+)季(?<episode>\d+)\s看完\s(?<logged_time>\d{4}-\d{2}-\d{2}\s.*)$'
  - '(?<name>.+)第(?<season>[0-9一二三四五六七八九十零百千]+)季\s看完\s(?<logged_time>\d{4}-\d{2}-\d{2}\s.*)$'
  - '(?<name>.+)第(?<season>[0-9一二三四五六七八九十零百千]+)季\s看完$'
  - '(?<name>.+)\sSeason\s(?<season>\d+)\s看完\s(?<logged_time>\d{4}-\d{2}-\d{2}\s.*)$'
  - '(?<name>.+)\sSeason\s(?<season>\d+)\s看完$'
finished_reg_pattern_list:
  - '[^\d\s]\s看完\s'
  - '\s看完$'
  - '\sSeason\s\d+\s看完\s'

max_thread_num: 12
min_task_num_per_thread: 1
```

| section name              | description                                                                                                                                                        |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| reg_pattern_list          | The regex list for parsing waching log, it will use one by one from the start, once matched, it will stop and use the matched result.                              |
| finished_reg_pattern_list | The regex list for determining if the watching is finished or not. It will be used on by one, once matched, it will stop and mark the watching as finished or not. |
| max_thread_num            | The max thread number for parsing the watching log.                                                                                                                |
| min_task_num_per_thread   | the min task number for a new thread to be created.                                                                                                                |

### regex

| name            | description                                    |
| --------------- | ---------------------------------------------- |
| name            | Necessary, the name of the watching            |
| season          | Optional, the season of the watching           |
| episode         | Optional, the episode of the watching          |
| time_at_episode | Optional, the time at the episode              |
| logged_time     | Optional, the time when the watching is logged |
| note            | Optional, any note for the watching            |

### caching

We treat each line of the watching log as one entry, we will parse each entry.

The caching combines each entry's hash value and the all regex list hash value as the caching key.

So once the regex list is changed, all entries will be re-parsed. If only some entries are changed, only those entries will be re-parsed.
