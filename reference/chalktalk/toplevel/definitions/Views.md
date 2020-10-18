# Coercions

The `Views:` is used to specify how a particular type can be viewed as another.  For example, a natural number can be viewed as an integer, and an integer can be viewed as a real number.

```yaml
[<id>]
Views: <target>
from: <command>
to: <command>
as: <clause>
using: <using-clause>+
written?: <text>
Metadata?: <metadata-form>
```

