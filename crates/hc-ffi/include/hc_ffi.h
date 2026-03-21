#ifndef HC_FFI_H
#define HC_FFI_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct HcString {
    char *ptr;
} HcString;

HcString hc_runtime_info_json(void);
void hc_string_free(HcString value);

#ifdef __cplusplus
}
#endif

#endif /* HC_FFI_H */
