import json

import networkx as nx
import matplotlib.pyplot as plt
import numpy as np
from matplotlib import colors

import data_process


def print_data(bc: data_process.Blockchain):
    sybil_node = "0xf1e9a125eccddc88d457010df3cedcc01c592ef3"
    print("sybil node miner times:", bc.get_miner_times(sybil_node))
    print("sybil node miner percentage:", bc.get_miner_percentage(sybil_node))


def print_graph():
    # 创建数据
    x = [1, 2, 3, 4]  # x轴从0.1到0.5
    y1 = [0.1, 0.0859, 0.0827, 0.0813]
    y2 = [0.2, 0.1850, 0.1811, 0.1745]
    y3 = [0.3, 0.2826, 0.2788, 0.2732]
    y4 = [0.4, 0.3819, 0.3734, 0.3703]
    y5 = [0.5, 0.4915, 0.4825, 0.4761]

    # 创建图形
    plt.figure(figsize=(8, 6), dpi=100)  # 设置图形大小和分辨率

    # 绘制三条曲线
    plt.plot(x, y1, label='10%Sybil', color='blue', linestyle='-', linewidth=2)
    plt.plot(x, y2, label='20%Sybil', color='red', linestyle='-', linewidth=2)
    plt.plot(x, y3, label='30%Sybil', color='green', linestyle='-', linewidth=2)
    plt.plot(x, y4, label='40%Sybil', color='green', linestyle='-', linewidth=2)
    plt.plot(x, y5, label='50%Sybil', color='green', linestyle='-', linewidth=2)

    # 设置坐标轴范围
    # plt.xlim(1, 4)
    # plt.ylim(0, 0.5)

    # 添加标题和坐标轴标签
    plt.title('Sybil Attack Stress Testing', fontsize=14)
    plt.xlabel('Number of Sybil Nodes', fontsize=12)
    plt.ylabel('Block Generation Probability of Sybil Nodes', fontsize=12)

    # 添加图例
    plt.legend(fontsize=10, loc='upper right')

    # 添加网格线
    plt.grid(True, linestyle='--', alpha=0.6)

    # 显示图形
    plt.tight_layout()  # 自动调整子图参数
    plt.show()


def print_graph_optimized():
    # Create data
    x = [1, 2, 3, 4, 5]
    y1 = [0.0951, 0.0859, 0.0827, 0.0813, None]
    y2 = [0.1963, 0.1850, 0.1811, 0.1745, None]
    y3 = [0.2943, 0.2826, 0.2788, 0.2732, None]
    y4 = [0.3978, 0.3819, 0.3734, 0.3703, None]
    y5 = [0.4916, 0.4888, 0.4825, 0.4761, None]

    # Ignore the last None value for plotting
    x_plot = x[:-1]
    y_data = [y1[:-1], y2[:-1], y3[:-1], y4[:-1], y5[:-1]]
    labels = ['10%', '20%', '30%', '40%', '50%']
    markers = ['o', 's', '^', 'D', 'v']
    linestyles = ['-', '--', '-.', ':', '-']

    # Create figure
    plt.figure(figsize=(8, 6), dpi=100)

    # Plot each series
    for y, label, marker, ls in zip(y_data, labels, markers, linestyles):
        plt.plot(x_plot, y, label=label, marker=marker,
                 linestyle=ls, linewidth=2, markersize=6)

    # Customize axes
    plt.xticks(x_plot)
    plt.ylim(0, 0.58)

    # Title and labels
    # plt.title('Sybil Attack Stress Test', fontsize=16, pad=15)
    plt.xlabel('Number of Sybil Nodes', fontsize=18)
    plt.ylabel('Block Generation Probability', fontsize=18)

    # Legend
    # plt.legend(fontsize=10, title_fontsize=12, loc='upper right')
    plt.legend(
        title='Real Stake Proportion',
        ncol=3,
        loc='upper right',
        fontsize=10,
        title_fontsize=12,
        # bbox_to_anchor=(0.35, 1.15),
        # frameon=False
    )
    # Grid
    plt.grid(True, linestyle='--', linewidth=0.5, alpha=0.7)

    # Layout
    plt.tight_layout()
    plt.show()


# 随机生成的网络模型对节点出块概率有一定的影响
# 一般需要1200以上的Block,出块概率才趋于稳定
# 总的来说,女巫攻击造成的传播链路的延长,会降低出块的概率
# 50Node 0.1Sybil 2Fake 2000Block  0.0859Rate
# 50Node 0.1Sybil 3Fake 1511Block  0.0827Rate
# 50Node 0.1Sybil 4Fake 1511Block  0.0813Rate
# 50Node 0.2Sybil 2Fake 1335Block  0.1850Rate
# 50Node 0.3Sybil 2Fake  874Block  0.2826Rate
# 50Node 0.4Sybil 2Fake 1729Block  0.3819Rate
# 50Node 0.5Sybil 2Fake 1752Block  0.4915Rate
if __name__ == '__main__':
    # bc = data_process.get_blockchain_from_json()
    print_graph_optimized()
