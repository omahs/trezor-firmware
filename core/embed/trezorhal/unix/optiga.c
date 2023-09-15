/*
 * This file is part of the Trezor project, https://trezor.io/
 *
 * Copyright (c) SatoshiLabs
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

#include "optiga.h"
#include <string.h>
#include "ecdsa.h"
#include "nist256p1.h"
#include "optiga_common.h"
#include "rand.h"

static const uint8_t DEVICE_CERT_CHAIN[] = {
    0x30, 0x82, 0x01, 0x90, 0x30, 0x82, 0x01, 0x37, 0xa0, 0x03, 0x02, 0x01,
    0x02, 0x02, 0x04, 0x4e, 0xe2, 0xa5, 0x0f, 0x30, 0x0a, 0x06, 0x08, 0x2a,
    0x86, 0x48, 0xce, 0x3d, 0x04, 0x03, 0x02, 0x30, 0x4f, 0x31, 0x0b, 0x30,
    0x09, 0x06, 0x03, 0x55, 0x04, 0x06, 0x13, 0x02, 0x43, 0x5a, 0x31, 0x1e,
    0x30, 0x1c, 0x06, 0x03, 0x55, 0x04, 0x0a, 0x0c, 0x15, 0x54, 0x72, 0x65,
    0x7a, 0x6f, 0x72, 0x20, 0x43, 0x6f, 0x6d, 0x70, 0x61, 0x6e, 0x79, 0x20,
    0x73, 0x2e, 0x72, 0x2e, 0x6f, 0x2e, 0x31, 0x20, 0x30, 0x1e, 0x06, 0x03,
    0x55, 0x04, 0x03, 0x0c, 0x17, 0x54, 0x72, 0x65, 0x7a, 0x6f, 0x72, 0x20,
    0x4d, 0x61, 0x6e, 0x75, 0x66, 0x61, 0x63, 0x74, 0x75, 0x72, 0x69, 0x6e,
    0x67, 0x20, 0x43, 0x41, 0x30, 0x1e, 0x17, 0x0d, 0x32, 0x32, 0x30, 0x34,
    0x33, 0x30, 0x31, 0x34, 0x31, 0x36, 0x30, 0x31, 0x5a, 0x17, 0x0d, 0x34,
    0x32, 0x30, 0x34, 0x33, 0x30, 0x31, 0x34, 0x31, 0x36, 0x30, 0x31, 0x5a,
    0x30, 0x0f, 0x31, 0x0d, 0x30, 0x0b, 0x06, 0x03, 0x55, 0x04, 0x03, 0x0c,
    0x04, 0x54, 0x32, 0x42, 0x31, 0x30, 0x59, 0x30, 0x13, 0x06, 0x07, 0x2a,
    0x86, 0x48, 0xce, 0x3d, 0x02, 0x01, 0x06, 0x08, 0x2a, 0x86, 0x48, 0xce,
    0x3d, 0x03, 0x01, 0x07, 0x03, 0x42, 0x00, 0x04, 0x9b, 0xbf, 0x06, 0xda,
    0xd9, 0xab, 0x59, 0x05, 0xe0, 0x54, 0x71, 0xce, 0x16, 0xd5, 0x22, 0x2c,
    0x89, 0xc2, 0xca, 0xa3, 0x9f, 0x26, 0x26, 0x7a, 0xc0, 0x74, 0x71, 0x29,
    0x88, 0x5f, 0xbd, 0x44, 0x1b, 0xcc, 0x7f, 0xa8, 0x4d, 0xe1, 0x20, 0xa3,
    0x67, 0x55, 0xda, 0xf3, 0x0a, 0x6f, 0x47, 0xe8, 0xc0, 0xd4, 0xbd, 0xdc,
    0x15, 0x03, 0x6e, 0xd2, 0xa3, 0x44, 0x7d, 0xfa, 0x7a, 0x1d, 0x3e, 0x88,
    0xa3, 0x41, 0x30, 0x3f, 0x30, 0x0e, 0x06, 0x03, 0x55, 0x1d, 0x0f, 0x01,
    0x01, 0xff, 0x04, 0x04, 0x03, 0x02, 0x00, 0x80, 0x30, 0x0c, 0x06, 0x03,
    0x55, 0x1d, 0x13, 0x01, 0x01, 0xff, 0x04, 0x02, 0x30, 0x00, 0x30, 0x1f,
    0x06, 0x03, 0x55, 0x1d, 0x23, 0x04, 0x18, 0x30, 0x16, 0x80, 0x14, 0x0a,
    0x80, 0xa4, 0x22, 0xc1, 0x6e, 0x36, 0x94, 0x24, 0xd3, 0xb8, 0x12, 0x67,
    0xb5, 0xaa, 0xe6, 0x46, 0x74, 0x74, 0xc8, 0x30, 0x0a, 0x06, 0x08, 0x2a,
    0x86, 0x48, 0xce, 0x3d, 0x04, 0x03, 0x02, 0x03, 0x47, 0x00, 0x30, 0x44,
    0x02, 0x20, 0x2f, 0xba, 0xc7, 0xe7, 0x99, 0x5b, 0x0e, 0x00, 0xc2, 0xc2,
    0xa7, 0xcb, 0xd8, 0xdd, 0x1d, 0x4e, 0xce, 0xf5, 0x58, 0x75, 0xa2, 0x65,
    0xc6, 0x86, 0xd4, 0xa8, 0xd2, 0x80, 0x68, 0x63, 0x5b, 0xf8, 0x02, 0x20,
    0x1e, 0xee, 0x62, 0x10, 0x0d, 0x0c, 0x97, 0x5a, 0x96, 0x34, 0x6f, 0x14,
    0xef, 0xe2, 0x6b, 0xc9, 0x3b, 0x00, 0xba, 0x30, 0x28, 0x9c, 0xe3, 0x7c,
    0xef, 0x17, 0x89, 0xbc, 0xc0, 0x68, 0xba, 0xb9, 0x30, 0x82, 0x01, 0xe0,
    0x30, 0x82, 0x01, 0x85, 0xa0, 0x03, 0x02, 0x01, 0x02, 0x02, 0x05, 0x00,
    0x94, 0x4e, 0x25, 0x7c, 0x30, 0x0a, 0x06, 0x08, 0x2a, 0x86, 0x48, 0xce,
    0x3d, 0x04, 0x03, 0x02, 0x30, 0x54, 0x31, 0x0b, 0x30, 0x09, 0x06, 0x03,
    0x55, 0x04, 0x06, 0x13, 0x02, 0x43, 0x5a, 0x31, 0x1e, 0x30, 0x1c, 0x06,
    0x03, 0x55, 0x04, 0x0a, 0x0c, 0x15, 0x54, 0x72, 0x65, 0x7a, 0x6f, 0x72,
    0x20, 0x43, 0x6f, 0x6d, 0x70, 0x61, 0x6e, 0x79, 0x20, 0x73, 0x2e, 0x72,
    0x2e, 0x6f, 0x2e, 0x31, 0x25, 0x30, 0x23, 0x06, 0x03, 0x55, 0x04, 0x03,
    0x0c, 0x1c, 0x54, 0x72, 0x65, 0x7a, 0x6f, 0x72, 0x20, 0x4d, 0x61, 0x6e,
    0x75, 0x66, 0x61, 0x63, 0x74, 0x75, 0x72, 0x69, 0x6e, 0x67, 0x20, 0x52,
    0x6f, 0x6f, 0x74, 0x20, 0x43, 0x41, 0x30, 0x20, 0x17, 0x0d, 0x32, 0x33,
    0x30, 0x31, 0x30, 0x31, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x5a, 0x18,
    0x0f, 0x32, 0x30, 0x35, 0x33, 0x30, 0x31, 0x30, 0x31, 0x30, 0x30, 0x30,
    0x30, 0x30, 0x30, 0x5a, 0x30, 0x4f, 0x31, 0x0b, 0x30, 0x09, 0x06, 0x03,
    0x55, 0x04, 0x06, 0x13, 0x02, 0x43, 0x5a, 0x31, 0x1e, 0x30, 0x1c, 0x06,
    0x03, 0x55, 0x04, 0x0a, 0x0c, 0x15, 0x54, 0x72, 0x65, 0x7a, 0x6f, 0x72,
    0x20, 0x43, 0x6f, 0x6d, 0x70, 0x61, 0x6e, 0x79, 0x20, 0x73, 0x2e, 0x72,
    0x2e, 0x6f, 0x2e, 0x31, 0x20, 0x30, 0x1e, 0x06, 0x03, 0x55, 0x04, 0x03,
    0x0c, 0x17, 0x54, 0x72, 0x65, 0x7a, 0x6f, 0x72, 0x20, 0x4d, 0x61, 0x6e,
    0x75, 0x66, 0x61, 0x63, 0x74, 0x75, 0x72, 0x69, 0x6e, 0x67, 0x20, 0x43,
    0x41, 0x30, 0x59, 0x30, 0x13, 0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d,
    0x02, 0x01, 0x06, 0x08, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07,
    0x03, 0x42, 0x00, 0x04, 0x9b, 0xf0, 0x76, 0x0c, 0xd7, 0x55, 0x5e, 0xfc,
    0x5b, 0x01, 0xe6, 0x4c, 0xe9, 0x46, 0x06, 0xcd, 0xee, 0xb4, 0x8a, 0x4f,
    0x91, 0x96, 0xf4, 0x54, 0xbc, 0x2a, 0x01, 0x41, 0xb3, 0x31, 0x88, 0x2d,
    0x06, 0x8b, 0x4b, 0x6e, 0x63, 0x79, 0x13, 0xdd, 0x22, 0x06, 0x54, 0xe2,
    0x8f, 0xde, 0x3c, 0x44, 0x21, 0x41, 0xf9, 0x53, 0xb3, 0xe3, 0x6a, 0xd9,
    0xa5, 0x75, 0x19, 0x00, 0x71, 0x19, 0xd9, 0xc9, 0xa3, 0x47, 0x30, 0x45,
    0x30, 0x0e, 0x06, 0x03, 0x55, 0x1d, 0x0f, 0x01, 0x01, 0xff, 0x04, 0x04,
    0x03, 0x02, 0x02, 0x04, 0x30, 0x12, 0x06, 0x03, 0x55, 0x1d, 0x13, 0x01,
    0x01, 0xff, 0x04, 0x08, 0x30, 0x06, 0x01, 0x01, 0xff, 0x02, 0x01, 0x00,
    0x30, 0x1f, 0x06, 0x03, 0x55, 0x1d, 0x23, 0x04, 0x18, 0x30, 0x16, 0x80,
    0x14, 0x28, 0xb2, 0x02, 0xf8, 0xf9, 0xc7, 0x8a, 0x74, 0xe8, 0xc1, 0x52,
    0xbb, 0xfb, 0x43, 0x3d, 0x99, 0xd0, 0xca, 0x03, 0xef, 0x30, 0x0a, 0x06,
    0x08, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x04, 0x03, 0x02, 0x03, 0x49, 0x00,
    0x30, 0x46, 0x02, 0x21, 0x00, 0x9b, 0x60, 0xfb, 0x5f, 0xc7, 0xee, 0x07,
    0x00, 0x31, 0x2d, 0x40, 0x2d, 0xcb, 0xe0, 0x77, 0xe4, 0x81, 0x82, 0xbb,
    0x94, 0x4b, 0xb1, 0xb3, 0x30, 0x79, 0xdd, 0xa0, 0xb6, 0xca, 0x9a, 0xb5,
    0xb7, 0x02, 0x21, 0x00, 0x81, 0x5d, 0xd9, 0x76, 0xab, 0xec, 0xb9, 0xb3,
    0xa8, 0x0f, 0x5b, 0x86, 0xb4, 0x49, 0x4f, 0xef, 0x94, 0xf2, 0xe0, 0xc1,
    0xf2, 0xc4, 0x5d, 0xe6, 0xec, 0x5f, 0x89, 0x5c, 0xa5, 0x6f, 0x04, 0x8b};

int optiga_sign(uint8_t index, const uint8_t *digest, size_t digest_size,
                uint8_t *signature, size_t max_sig_size, size_t *sig_size) {
  const uint8_t DEVICE_PRIV_KEY[32] = {1};

  if (index != OPTIGA_DEVICE_ECC_KEY_INDEX) {
    return false;
  }

  if (max_sig_size < 72) {
    return OPTIGA_ERR_SIZE;
  }

  uint8_t raw_signature[64] = {0};
  int ret = ecdsa_sign_digest(&nist256p1, DEVICE_PRIV_KEY, digest,
                              raw_signature, NULL, NULL);
  if (ret != 0) {
    return OPTIGA_ERR_CMD;
  }

  *sig_size = ecdsa_sig_to_der(raw_signature, signature);
  return OPTIGA_SUCCESS;
}

bool optiga_cert_size(uint8_t index, size_t *cert_size) {
  if (index != OPTIGA_DEVICE_CERT_INDEX) {
    return false;
  }

  *cert_size = sizeof(DEVICE_CERT_CHAIN);
  return true;
}

bool optiga_read_cert(uint8_t index, uint8_t *cert, size_t max_cert_size,
                      size_t *cert_size) {
  if (index != OPTIGA_DEVICE_CERT_INDEX) {
    return false;
  }

  if (max_cert_size < sizeof(DEVICE_CERT_CHAIN)) {
    return false;
  }

  memcpy(cert, DEVICE_CERT_CHAIN, sizeof(DEVICE_CERT_CHAIN));
  *cert_size = sizeof(DEVICE_CERT_CHAIN);
  return true;
}

bool optiga_random_buffer(uint8_t *dest, size_t size) {
  random_buffer(dest, size);
  return true;
}
