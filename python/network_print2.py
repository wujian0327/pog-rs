import json

import networkx as nx
import matplotlib.pyplot as plt
import numpy as np
from matplotlib import colors

import data_process


def print_graph(bc: data_process.Blockchain,
                json_file: str = "../graph.json",
                output_path: str = '../figures/graph.png'):
    """
    生成仿红蓝热图风格的区块链传播网络图
    """
    with open(json_file, 'r') as file:
        data = json.load(file)

    G = nx.Graph()
    for u, v in data:
        G.add_edge(u, v)

    # 计算权重
    edge_weights = {}
    for u, v in G.edges():
        w = bc.edge_path.get(f"{u}>{v}", 0) + bc.edge_path.get(f"{v}>{u}", 0)
        edge_weights[(u, v)] = w
    weights = np.array(list(edge_weights.values()), dtype=float)
    if weights.max() == 0:
        norm_w = weights
    else:
        norm_w = (weights - weights.min()) / (weights.max() - weights.min())

    # 颜色映射：冷—>热, 蓝到红
    cmap = plt.cm.coolwarm
    edge_colors = [cmap(val) for val in norm_w]

    # 宽度映射
    min_w, max_w = 0.5, 4.0
    widths = min_w + (max_w - min_w) * norm_w

    pos = nx.spring_layout(G, k=1.2, iterations=800, seed=42)

    plt.figure(figsize=(10, 10), dpi=300)
    ax = plt.gca()

    # 节点出块越多，节点越大
    node_sizes = [300 * (1 + bc.get_miner_percentage(n) * 20) for n in G.nodes()]

    # 节点网络贡献越多，颜色越鲜艳
    node_colors = [bc.get_node_path_percentage(n) for n in G.nodes()]
    arr = np.array(node_colors)
    node_colors = np.log(arr)

    # 绘制节点
    nodes = nx.draw_networkx_nodes(
        G,
        pos,
        node_size=node_sizes,
        node_color=node_colors,
        cmap=cmap,
        edgecolors="#444444",  # 节点边框颜色
        linewidths=0.8,
        alpha=0.95,
    )

    # 画无向边骨架
    nx.draw_networkx_edges(
        G, pos,
        alpha=0.1,
        width=0.5,
        edge_color='#999999',
        ax=ax
    )
    # 画红蓝权重边
    nx.draw_networkx_edges(
        G, pos,
        edgelist=list(edge_weights.keys()),
        width=widths,
        edge_color=edge_colors,
        ax=ax
    )

    # 添加标题
    # ax.set_title('Network Contribution Percentage', fontsize=18, pad=20)

    # 添加边权重颜色条（侧边栏）
    # norm = colors.Normalize(vmin=weights.min(), vmax=weights.max())
    # sm = plt.cm.ScalarMappable(cmap=cmap, norm=norm)
    # sm._A = []  # dummy array for the mappable
    # cbar = plt.colorbar(sm, ax=ax, fraction=0.025, pad=0.02)
    cbar = plt.colorbar(nodes, fraction=0.025, pad=0.02)  # 颜色条说明
    contrib = [bc.get_node_path_percentage(n) for n in G.nodes()]
    ticks = np.linspace(min(contrib), max(contrib), 7)
    cbar.set_ticklabels([f"{t * 100:.2f}%" for t in ticks])
    # cbar.set_label('Contribution Percentage', fontsize=14)

    ax.margins(0.02)
    plt.axis('off')
    plt.tight_layout()
    # plt.show()
    plt.savefig(output_path)
    plt.close()


if __name__ == '__main__':
    bc = data_process.get_blockchain_from_json()
    # 示例使用
    print_graph(bc)
