#pragma pattern_limit 9999999999
#pragma array_limit 9999999999
#pragma endian little


struct PascalString {
    u8 len;
    char data[len];
};


bitfield LabelHeader {
    label_idx : 5;
    size_tag  : 2;
    is_ref    : 1;
} [[static]];

struct Label {
    u8 first [[hidden]];

    bool inplace = ((first & (1 << 7)) == 0) [[export]];

    if (inplace) {
        // InPlace label
        char label[first];
    } else {
    
        u8 label_idx = first & (255 >> 3) [[export]];
        u8 size_tag = (first & 96) >> 5;
    
        // Reference label
        if (size_tag == 0) {
            u8 entry_idx;
        } else if (size_tag == 1) {
            u16 entry_idx;
        } else if (size_tag == 2) {
            u32 entry_idx;
        } else {
            u64 entry_idx;
        }
    }
};

struct Entry {
    u8 scope_id;
    u8 label_count;
    Label labels[label_count];
};

struct Meta {
    u32 chunk_size;
    u8 scope_count;
    PascalString scopes[scope_count];
};

struct Index {
    char magic[5];          // "ixx02"
    Meta meta;
    u32 entry_count;
    Entry entries[entry_count];
};

Index index @ 0x0;