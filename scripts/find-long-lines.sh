#!/usr/bin/env bash

# do not stop on first error
# so a non-zero exit code can be processed
set +e

# search for long lines
grep -l -E '.{101}' `find . -iname '*.go'`

EXIT="$?"

if [[ "$EXIT" == "1" ]]
then
  # if the exit code of grep is 1 then no matches
  # where found, which means no long lines were
  # found and so report an exit code of 0 since
  # finding no long lines is a success
  exit 0
fi

# otherwise, the exit code for grep is 0 or >1
# 0 indicates one or more lines were selected
# >1 indicates an error occured
# in either case, report 1 to indicate an error
exit 1
