# Usage

```python
from leveldb import Database, OpenOptions, ReadOptions

path = "path/to/db"

db = Database(path, open_options=OpenOptions())

for key, value in db.iter(read_options=ReadOptions()):
    print(f"{key}: {value}")
```