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


def print_graph_optimized():
    # Create data
    miner_ratios = np.arange(5, 55, 5)
    ntd_values = np.array([3, 3, 3, 3, 3, 3, 4, 4, 4, 4])
    reward_values = np.array([100, 95, 88, 80, 70, 65, 61, 55, 50, 47])

    # 创建图形和双轴
    fig, ax1 = plt.subplots(figsize=(8, 6))

    # 绘制NTD曲线（左Y轴）
    color = 'tab:red'
    ax1.set_xlabel('Selfish Miner Ratio', fontsize=18)
    ax1.set_ylabel('NTD', fontsize=18)
    ax1.plot(miner_ratios, ntd_values, color=color, marker='o', linestyle='--', linewidth=2, label='NTD')
    # ax1.tick_params(axis='y', labelcolor=color)
    ax1.set_ylim(2, 5)  # 聚焦NTD变化范围
    ax1.yaxis.set_major_locator(plt.MaxNLocator(integer=True))

    # 设置x轴刻度带百分号
    ax1.set_xticks(miner_ratios)
    ax1.set_xticklabels([f"{x}%" for x in miner_ratios])  # 添加%符号

    # 创建第二个Y轴
    ax2 = ax1.twinx()
    color = 'tab:blue'
    ax2.set_ylabel('Total Reward', fontsize=18)
    ax2.plot(miner_ratios, reward_values, color=color, marker='s', linestyle='-', linewidth=2, label='Reward')
    ax2.tick_params(axis='y')

    # 添加标题和图例
    # plt.title('Impact of Selfish Miner Ratio on NTD and Rewards (50-node Network)', fontsize=14, pad=20)
    lines1, labels1 = ax1.get_legend_handles_labels()
    lines2, labels2 = ax2.get_legend_handles_labels()
    ax1.legend(lines1 + lines2, labels1 + labels2, loc='upper center', ncol=2)

    # 调整布局
    fig.tight_layout()
    plt.grid(alpha=0.3)
    plt.show()


if __name__ == '__main__':
    print_graph_optimized()
