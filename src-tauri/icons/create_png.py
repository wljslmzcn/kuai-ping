import struct
import zlib

def create_png(width, height, filename):
    # PNG 文件签名
    signature = b'\x89PNG\r\n\x1a\n'
    
    # IHDR 块
    ihdr_data = struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0)
    ihdr_crc = zlib.crc32(b'IHDR' + ihdr_data) & 0xffffffff
    ihdr = struct.pack('>I', 13) + b'IHDR' + ihdr_data + struct.pack('>I', ihdr_crc)
    
    # IDAT 块 (像素数据)
    raw_data = b''
    for y in range(height):
        raw_data += b'\x00'  # 滤波器类型 (None)
        for x in range(width):
            # 蓝色渐变
            r = 64
            g = min(255, 158 + (y * 10))
            b = 255
            a = 255
            raw_data += struct.pack('BBBB', r, g, b, a)
    
    compressed = zlib.compress(raw_data)
    idat_crc = zlib.crc32(b'IDAT' + compressed) & 0xffffffff
    idat = struct.pack('>I', len(compressed)) + b'IDAT' + compressed + struct.pack('>I', idat_crc)
    
    # IEND 块
    iend_crc = zlib.crc32(b'IEND') & 0xffffffff
    iend = struct.pack('>I', 0) + b'IEND' + struct.pack('>I', iend_crc)
    
    with open(filename, 'wb') as f:
        f.write(signature + ihdr + idat + iend)
    
    print(f"Created {filename}")

# 创建不同尺寸
create_png(32, 32, '32x32.png')
create_png(128, 128, '128x128.png')
create_png(256, 256, '128x128@2x.png')
