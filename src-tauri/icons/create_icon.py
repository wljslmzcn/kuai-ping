import struct

# 创建一个简单的 16x16 ICO 文件
def create_ico():
    # ICO 文件头
    header = struct.pack('<HHH', 0, 1, 1)  # 保留, 类型(1=ICO), 数量
    
    # 图标目录项 (16x16, 32bpp)
    dir_entry = struct.pack('<BBBBHHII', 
        16,  # 宽度
        16,  # 高度
        0,   # 颜色数
        0,   # 保留
        1,   # 颜色平面
        32,  # 每像素位数
        16*16*4 + 16*16//8,  # 图像数据大小
        22   # 数据偏移
    )
    
    # 像素数据 (BGRA, 蓝色主题)
    pixels = b''
    for y in range(16):
        for x in range(16):
            # 创建一个简单的蓝色渐变图案
            r = 64
            g = 158 + (y * 5)
            b = 255
            a = 255
            if g > 255:
                g = 255
            pixels += struct.pack('BBBB', b, g, r, a)
    
    # AND 掩码 (全部不透明)
    and_mask = b'\x00' * (16 * 16 // 8)
    
    with open('icon.ico', 'wb') as f:
        f.write(header + dir_entry + pixels + and_mask)
    
    print("Created icon.ico")

if __name__ == '__main__':
    create_ico()
