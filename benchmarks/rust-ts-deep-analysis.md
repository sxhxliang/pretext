# Rust vs TS deep analysis

- rows: 108
- lineCount mismatch rows: 0
- line.text mismatch rows: 55
- cursor(start/end) mismatch rows: 101
- execution error rows: 0

## Dominant patterns
- URL/query boundary related rows: 9
- CJK/emoji boundary related rows: 2
- cursor segment-start drift rows: 101

## By case
- mixed-app (normal): lineCountMismatch 0/12, lineTextMismatch 11/12, cursorMismatch 12/12, executionError 0/12
- arabic-punct (normal): lineCountMismatch 0/12, lineTextMismatch 2/12, cursorMismatch 12/12, executionError 0/12
- arabic-mark (normal): lineCountMismatch 0/12, lineTextMismatch 12/12, cursorMismatch 12/12, executionError 0/12
- myanmar (normal): lineCountMismatch 0/12, lineTextMismatch 10/12, cursorMismatch 12/12, executionError 0/12
- cjk-kinsoku (normal): lineCountMismatch 0/12, lineTextMismatch 5/12, cursorMismatch 12/12, executionError 0/12
- emoji-zwj (normal): lineCountMismatch 0/12, lineTextMismatch 9/12, cursorMismatch 12/12, executionError 0/12
- soft-hyphen (normal): lineCountMismatch 0/12, lineTextMismatch 5/12, cursorMismatch 5/12, executionError 0/12
- pre-wrap-spaces (pre-wrap): lineCountMismatch 0/12, lineTextMismatch 0/12, cursorMismatch 12/12, executionError 0/12
- pre-wrap-tabs (pre-wrap): lineCountMismatch 0/12, lineTextMismatch 1/12, cursorMismatch 12/12, executionError 0/12

## Top mismatching rows
- mixed-app @ 80px: lineCountDelta=0, lineTextMismatchCount=8, cursorMismatchCount=11, firstLineTextDiff=i=1, ts="界 👋🏽 ", rust="世界 👋🏽 ", firstCursorDiff=i=1, ts=(3,0)->(7,0), rust=(2,0)->(6,0)
- mixed-app @ 100px: lineCountDelta=0, lineTextMismatchCount=8, cursorMismatchCount=10, firstLineTextDiff=i=2, ts="https://ex", rust="https://", firstCursorDiff=i=0, ts=(0,0)->(5,0), rust=(0,0)->(4,0)
- mixed-app @ 120px: lineCountDelta=0, lineTextMismatchCount=8, cursorMismatchCount=8, firstLineTextDiff=i=0, ts="Hello 世界 ", rust="Hello 世界 👋🏽 ", firstCursorDiff=i=0, ts=(0,0)->(5,0), rust=(0,0)->(6,0)
- mixed-app @ 140px: lineCountDelta=0, lineTextMismatchCount=5, cursorMismatchCount=6, firstLineTextDiff=i=1, ts="https://exampl", rust="https://", firstCursorDiff=i=0, ts=(0,0)->(7,0), rust=(0,0)->(6,0)
- mixed-app @ 160px: lineCountDelta=0, lineTextMismatchCount=5, cursorMismatchCount=6, firstLineTextDiff=i=1, ts="https://example.c", rust="https://example.", firstCursorDiff=i=0, ts=(0,0)->(7,0), rust=(0,0)->(6,0)
- mixed-app @ 180px: lineCountDelta=0, lineTextMismatchCount=4, cursorMismatchCount=6, firstLineTextDiff=i=2, ts="/path?", rust="/path?q=alpha&lang=", firstCursorDiff=i=0, ts=(0,0)->(7,0), rust=(0,0)->(6,0)
- mixed-app @ 200px: lineCountDelta=0, lineTextMismatchCount=4, cursorMismatchCount=5, firstLineTextDiff=i=1, ts="https://example.com/p", rust="https://example.com/", firstCursorDiff=i=0, ts=(0,0)->(7,0), rust=(0,0)->(6,0)
- mixed-app @ 220px: lineCountDelta=0, lineTextMismatchCount=3, cursorMismatchCount=4, firstLineTextDiff=i=1, ts="https://example.com/pat", rust="https://example.com/", firstCursorDiff=i=0, ts=(0,0)->(7,0), rust=(0,0)->(6,0)
- mixed-app @ 240px: lineCountDelta=0, lineTextMismatchCount=0, cursorMismatchCount=4, firstCursorDiff=i=0, ts=(0,0)->(7,0), rust=(0,0)->(6,0)
- mixed-app @ 260px: lineCountDelta=0, lineTextMismatchCount=3, cursorMismatchCount=4, firstLineTextDiff=i=1, ts="https://example.com/path?", rust="https://example.com/path?q=", firstCursorDiff=i=0, ts=(0,0)->(7,0), rust=(0,0)->(6,0)
- mixed-app @ 300px: lineCountDelta=0, lineTextMismatchCount=3, cursorMismatchCount=4, firstLineTextDiff=i=1, ts="https://example.com/path?", rust="https://example.com/path?q=alpha", firstCursorDiff=i=0, ts=(0,0)->(7,0), rust=(0,0)->(6,0)
- mixed-app @ 320px: lineCountDelta=0, lineTextMismatchCount=3, cursorMismatchCount=4, firstLineTextDiff=i=1, ts="https://example.com/path?", rust="https://example.com/path?q=alpha&", firstCursorDiff=i=0, ts=(0,0)->(7,0), rust=(0,0)->(6,0)
