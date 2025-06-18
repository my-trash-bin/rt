#ifndef JSONC_H
#define JSONC_H

#ifdef __cplusplus
#include <cstddef>
extern "C" {
#else
#include <stdbool.h>
#include <stddef.h>
#endif

typedef enum jsonc_value_type {
  JSONC_VALUE_TYPE_NULL,
  JSONC_VALUE_TYPE_BOOLEAN,
  JSONC_VALUE_TYPE_NUMBER,
  JSONC_VALUE_TYPE_STRING,
  JSONC_VALUE_TYPE_ARRAY,
  JSONC_VALUE_TYPE_OBJECT,
} jsonc_value_type;

typedef struct jsonc_value jsonc_value;

typedef struct jsonc_array {
  jsonc_value *values;
  size_t count;
} jsonc_array;

typedef struct jsonc_object_entry jsonc_object_entry;

typedef struct jsonc_object {
  jsonc_object_entry *entries;
  size_t count;
} jsonc_object;

struct jsonc_value {
  jsonc_value_type type;
  union {
    bool boolean;
    double number;
    char *string;
    jsonc_object object;
    jsonc_array array;
  } value;
};

struct jsonc_object_entry {
  char *key;
  jsonc_value value;
};

typedef bool err_t;

err_t jsonc_parse(const char *source, jsonc_value *out, bool *out_is_error);
void jsonc_free(jsonc_value value);

#ifdef __cplusplus
}
#endif

#endif
