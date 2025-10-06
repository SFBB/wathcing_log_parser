# Working Example Demonstration

## Sample Configuration (sample_config.yaml)
```yaml
reg_pattern_list:
  - "(?P<name>[^\\s]+)"
finished_reg_pattern_list:
  - "finished"
```

## Sample Log File (sample_log.txt)
```
Show1 Episode 1
Show2 Episode 2 finished
Show1 Episode 3  
Show3 Episode 1 finished
```

## Application Output
```bash
$ cargo run -- -f sample_log.txt -c sample_config.yaml
name: Show1
```

## Analysis
The application correctly:
1. Parsed the log file using regex patterns
2. Identified show names from each line
3. Detected which shows are marked as "finished"
4. Reported only unfinished shows (Show1 appears twice but never finished)

This demonstrates the core functionality is working, despite the implementation issues identified in the analysis.