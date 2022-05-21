# MLDL
Parser for ML binaries.

## Sections

### `.mldyn`

| Name      | Type       |
|-----------|------------|
| Function  | `void*`    |
| Symbol    | `uint8_t*` |
| Record    | `uint8_t*` |

- **Function**: local stub/fallback.
- **Symbol**: target symbol.
- **Record**: function record (could be null).

### `.mlhook`

| Name      | Type               |
|-----------|--------------------|
| Target    | `void*`/`uint8_t*` |
| Hook      | `void*`            |
| Flags     | `uint64_t`         |

#### Flags

Bit 0 - **Dispatcher (internal):** set when `Hook` represents a pointer to a dispatcher.
Bit 1 - **Dynamic (internal):** set when `Target` represents a symbol for a dynamic target.
Bit 2 - **Locking:** When this flag is set, the hook will lock the chain.
Bit 3 - **Preload:** When this flag is set, the hook will be loaded automatically.
Bit 4 - **Optional:** When this and the preload flag are set, preload failures will not stop the module loading process.
Bit 5 - **Priority:** When this flag is set, the hook will take the highest priority on the chain.
Bit 6-63 - Reserved.