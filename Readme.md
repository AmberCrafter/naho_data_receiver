# Metadata
 - version: dev

# Structure
1. csv data seperate by date
2. sqlite data seperate by date


# config
```config.json
{
    global: {
    },
    data: [ // in-order list
        {
            name: String,
            sqlite_name: String,
            datatype: {
                rust: String,
                sqlite: String,
            },
            regexp: String
        },...
    ]
}

```

# Workflow
1. Receive data
2. dispatch (mpsc)
 - logger
    - system log: all
    - data log: ~7D
    - error log: all
 - csv data
 - sqlite data
 - database uploader

