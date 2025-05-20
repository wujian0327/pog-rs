from matplotlib import pyplot as plt
import seaborn as sns


def block_size():
    indices = [i for i in range(50)]
    bls = [604, 786, 828, 870, 912, 954, 996, 1038, 1080, 1122, 1164, 1206, 1248, 1290, 1332, 1374, 1416, 1458, 1500,
           1542,
           1584, 1626, 1668, 1710, 1752, 1794, 1836, 1878, 1920, 1962, 2004, 2046, 2088, 2130, 2172, 2214, 2256, 2298,
           2340,
           2382, 2424, 2466, 2508, 2550, 2592, 2634, 2676, 2718, 2760, 2802]
    bls_zstd_compress = [642, 749, 777, 802, 825, 849, 872, 891, 917, 936, 961, 982, 1006, 1024, 1048, 1065, 1086, 1107,
                         1125, 1152, 1172, 1193, 1216, 1232, 1254, 1272, 1293, 1319, 1333, 1359, 1378, 1397, 1425, 1449,
                         1459, 1488, 1505, 1533, 1551, 1563, 1586, 1614, 1631, 1645, 1676, 1691, 1712, 1734, 1757, 1786]
    secp256k1 = [562, 910, 1084, 1258, 1432, 1606, 1780, 1954, 2128, 2302, 2476, 2650, 2824, 2998, 3172, 3346, 3520,
                 3694,
                 3868, 4042, 4216, 4390, 4564, 4738, 4912, 5086, 5260, 5434, 5608, 5782, 5956, 6130, 6304, 6478, 6652,
                 6826,
                 7000, 7174, 7348, 7522, 7696, 7870, 8044, 8218, 8392, 8566, 8740, 8914, 9088, 9262]

    # 绘制折线图
    plt.figure(figsize=(8, 5))
    plt.plot(indices, bls,
             linestyle='-',
             linewidth=2.5,
             color='#2c7bb6',
             label='bls',
             alpha=0.9)
    plt.plot(indices, bls_zstd_compress,
             linestyle='-',
             linewidth=2.5,
             color='#72b036',
             label='bls_zstd_compress',
             alpha=0.9)
    plt.plot(indices, secp256k1,
             linestyle='-',
             linewidth=2.5,
             color='#d7191c',
             label='Secp256k1',
             alpha=0.9)

    # 添加标题和标签
    plt.title('Path Tracing Block Size Comparison')
    plt.xlabel('Number of Paths')
    plt.ylabel('Block Size (Bytes)')
    plt.legend()
    plt.grid(True)

    # 显示图表
    plt.show()


def verify_time():
    indices = [i for i in range(50)]
    bls = [0, 1201, 896, 1047, 1169, 1168, 1369, 1256, 1391, 1528, 1677, 1696, 1374, 1526, 1805, 1542, 1581, 1601, 1597,
           1884, 2062, 2149, 1981, 2528, 3217, 2667, 3290, 2345, 2745, 2885, 2784, 2661, 4268, 2749, 2719, 3874, 2468,
           2681, 3264, 3196, 4072, 3412, 3457, 2757, 4238, 3725, 2790, 2968, 2585, 3350]
    bls_with_decompress = [0, 1441, 1002, 1245, 1215, 1187, 1490, 1503, 1806, 1401, 1395, 1775, 1694, 1720, 1775, 1739,
                           1701, 2112, 1864, 1996, 2651, 2209, 2807, 3517, 2272, 3158, 2502, 2974, 2801, 3934, 3129,
                           2537, 3559, 3533, 2684, 2797, 3121, 5243, 2744, 3226, 3217, 3778, 5352, 3308, 4942, 3526,
                           3995, 4066, 3825, 4177]
    secp256k1 = [0, 725, 1314, 2011, 2528, 3325, 3913, 4557, 5106, 5919, 6283, 7331, 7487, 8428, 8909, 9463, 10189,
                 10537, 11280, 11687, 13345, 13828, 13719, 14283, 15404, 16051, 16140, 16731, 17140, 18670, 18880,
                 19549, 20618, 20932, 22164, 22259, 23128, 23160, 23459, 25028, 25137, 26462, 26637, 27620, 27865,
                 28135, 30603, 29699, 29839, 32766]

    # 设置样式
    # sns.set_theme(style="whitegrid")
    plt.figure(figsize=(8, 5))
    # 绘制折线图（移除了数据点标记）
    plt.plot(indices, bls,
             linestyle='-',
             linewidth=2.5,
             color='#2c7bb6',
             label='bls',
             alpha=0.9)
    plt.plot(indices, bls_with_decompress,
             linestyle='-',
             linewidth=2.5,
             color='#72b036',
             label='bls_zstd_decompress',
             alpha=0.9)
    plt.plot(indices, secp256k1,
             linestyle='-',
             linewidth=2.5,
             color='#d7191c',
             label='Secp256k1',
             alpha=0.9)
    # 装饰样式
    plt.title('Path Tracing Verify Algorithm Comparison', )
    plt.xlabel('Number of Paths')
    plt.ylabel('Verify Time (Microseconds)', )
    # 自动优化坐标轴范围
    # plt.ylim(0, max(secp256k1 + bls) * 1.1)
    # 图例美化
    # plt.legend(loc='upper left',
    #            frameon=True,
    #            fontsize=12,
    #            shadow=True,
    #            borderpad=1)
    plt.legend()
    plt.grid(True)
    plt.show()


if __name__ == '__main__':
    block_size()
    verify_time()
