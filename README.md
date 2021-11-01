# cw_parser

Cloudwatch logs will be formatted differently depending on what runtime you are using. 
This library should make things abit easier.

Node:
```2020-11-18T23:52:30.128Z  6e48723a-1596-4313-a9af-e4da9214d637  INFO  {"data": "Hello World"}```

Python:
```[INFO]	2020-11-18T23:52:30.128Z	6e48723a-1596-4313-a9af-e4da9214d637	{"data": "Hello World"}```

Dotnet/Provided Runtimes _just logs the raw message from the runtime_:
```{"data": "Hello World"}```


