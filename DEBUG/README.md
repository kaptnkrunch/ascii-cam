# Debug Work Directory

*Use this directory for debugging tasks and proposed fixes*

---

## Purpose

This folder is for debug agent work. Place proposed fixes, debug notes, and investigation results here.

---

## Structure

```
DEBUG/
├── fix/
│   └── {issue-name}/
│       └── Proposed fix files
└── notes.md
```

---

## Fix Workflow

1. Create `DEBUG/fix/{name}/` for proposed fix
2. Test: `[BUILD_COMMAND_DEBUG] && [BUILD_COMMAND_RELEASE]`
3. If successful, merge to `src/`
4. Document in TROUBLESHOOTING.md

---

*Last Updated: [date]*
