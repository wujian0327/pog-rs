import copy
import sys

import zstandard as zstd
import base64

# 假设以太坊地址列表
addresses = ["0x1ce7b039c2866587e59ac747799960c56b59625c", "0xbc6113df9d9fd7949014942826ff18e275b14e79",
             "0xdf1de705c86fbc531e156c918ad3340a93c5982f", "0x3bdca8bef0960f0de46797c9c473d51539986d30",
             "0x253f108cf961765b48ca9f7478810f30d8247e73", "0xcf6af967562c8dec50c0b988ec75bc8ddb220678",
             "0xdba171dc21bb691845d4862366fe3922eee9d84c", "0x07c7169ba5b5ccd276d942a18317a96e86592240",
             "0x72883a9f152b8fc8af922ad48349b2cd8e7ea63d", "0x634365b9cd5013caf95d4b0f6c24aad442c2f1d0",
             "0x220cef4c7ed331382f8e8afbb3bd9540853fc6b6", "0x2ec3b2d2f59ebd545dfdf2adabb88aa357e70ad3",
             "0xe44983a2c1da287c14094cf84f9ef8fb89ead8c6", "0x9c6454256c364f092d12840f1e9bb19b50071c8a",
             "0xd24e79896449fe51f189bd13aa61acb5e5bd8503", "0x5e5441a0e4d5e66edf4fe013a45acef7d182500d",
             "0xdbf446a503da09887e8880160939f850f0a92279", "0xe63715a7f7c23093a2fd49a09b876c3414d4f6ab",
             "0xf3a8486c120a9daae36321f3cba3576ae4b3e243", "0xa7ed39ffb8185f2b4d980b242dc78d29b31de268",
             "0x08df68754dc84312f7d00584b11b46515cac44f4", "0x641dcaac117fd2b9041910a080349962bc41bde4",
             "0x52c529c59d418139898293c1f85b33bb986c667f", "0xf1f86a68b53bcaf54626e174a0961fa6ba7144cf",
             "0x2e78d55f98019ef9f25f7b5028240e51686a1336", "0x0327dd04899c3fca0854e66455a6e44834e19a71",
             "0x52476c94915d456759b08082ecf612ec7a31dd4c", "0x5a6c24912d41010d7244c21fbc3729216fe993a7",
             "0x7e33dde061b13d810cd5d45596e24ad6fca38639", "0x3ad4fdc80bdf08c73be2c0b9b7589219bc266915",
             "0x17417e3d2f78ec1190eaf706f51f30955de20361", "0xe85458c1a6bfb759a4981be4ae228ed72104f7ac",
             "0x92c4846308641e7e1f779ff9339ca4c2c8f47231", "0x66da6924307cf91cbf1d421667398bbba920821b",
             "0x5d21d4eae2fe3eb4402fadf6d6c362b448f5d3d7", "0x8bb150ea4c997a8880171fe6376aa78391878260",
             "0x2b901ce92727f9912552c73c69f3bfeccad9eab2", "0xe35ddd5ff103f9527f4a90e480d960eb5b5dfa3f",
             "0xc0496ee2dd6484c673b29c425b0bd1a517a7f75e", "0xbe091df7eb50dcc5f7b899f15cc09091ffd2fcb6",
             "0xf02a315724f9d52bf9a5c2eb0aa9548ba832d420", "0x454e84e9d2997468c3fa58c6841c351b591c2ac6",
             "0x27a855e76d0e91128a052b479df1e4fde065e177", "0xaf5893087901b12326c94d6f21874f3261833703",
             "0x98a8286c62d844c0c278cdd0bf2809151b83c986", "0x059053b532a5b7fabda627f41930c345c608b219",
             "0xd6036d15fc61f04af6820680ea9c1a85bab7f765", "0xa737fe84a56759b0e3a7b02ebb2aec0681ba78bd",
             "0x05e33ee4fcac57d8d33a64f64ed2a573c162110d", "0xbdfe8d6eb695dbf7fc14a8bbc8aafbc00a74fb51",
             "0x33f26dd31c098b869833416642f86ef82f50cb00", "0xc47406d4983ecd838d00910d2bcb899e67411bb1"]
cloned_addresses = copy.copy(addresses)
# Step 1: 转换为二进制块
binary_data = b""
for addr in addresses:
    hex_part = addr[2:]  # 去掉0x
    binary_data += bytes.fromhex(hex_part)
# Step 2: 压缩（使用Zstd）
cctx = zstd.ZstdCompressor(level=22)
compressed_data = cctx.compress(binary_data)
# origin_size = sum(sys.getsizeof(s) for s in addresses)
origin_size = sum(len(s.encode('utf-8')) for s in addresses)
print("origin size:", origin_size)
print("compressed size:", len(compressed_data))

# 解压数据
dctx = zstd.ZstdDecompressor()
binary_data = dctx.decompress(compressed_data)
# 恢复地址
addresses = []
for i in range(len(cloned_addresses)):
    start = i * 20
    end = start + 20
    eth_bytes = binary_data[start:end]
    hex_str = eth_bytes.hex()
    addresses.append("0x" + hex_str)
print("Recovered addresses:", addresses)
