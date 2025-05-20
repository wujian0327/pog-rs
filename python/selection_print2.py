import matplotlib
import numpy as np
import matplotlib.pyplot as plt
from matplotlib import cm
from matplotlib.patches import Patch, FancyArrowPatch

"""
Equal‐Sharing Strategy ->2
在此策略下，其余节点将真实权益和网络贡献度——即$1-S$与$1-C$——在所有节点间平均分配，产生最小的总虚拟权益。

Collusive‐Maximization Strategy ->3
在此策略下，其余节点可联合行动，集中部分资源以最大化它们的总虚拟权益，从而对单个节点的出块概率造成最大的削弱。
"""


def block_probability(S, C, K=2, strategy=2):
    s_real = 1000 * S
    phi_s = np.where(S > 0.5, 0, 0.5 - S)
    s_v = s_real * (1 + K * C * phi_s)
    if strategy == 1:
        # 自己可取得的最大虚拟权益
        # 因为自己的真实权益少于50%,找一个50%的节点，把剩余网络贡献度都给它
        # 则需要2个节点
        s_real_node_1 = 1000 * 0.5
        s_v__node_1 = s_real_node_1
        s_real_node_2 = 1000 * (1 - 0.5 - S)
        s_v__node_2 = s_real_node_2
        s_v_total = s_v + s_v__node_1 + s_v__node_2
        return s_v / s_v_total
    elif strategy == 2:
        # 其他节点平均分配策略
        sum_max = 0
        s_v_rest = None
        for i in range(1, 100):
            s_real_one = 1000 * (1 - S) / i
            phi_s_i = np.where((1 - S) / i > 0.5, 0, 0.5 - (1 - S) / i)
            s_v_one = s_real_one * (1 + K * (1 - C) / i * phi_s_i)
            total = np.sum(s_v_one * i)
            if total > sum_max:
                sum_max = total
                s_v_rest = s_v_one * i
        s_v_total = s_v + s_v_rest
        return s_v / s_v_total
    elif strategy == 3:
        # 其他节点串谋使节点A虚拟权益比值最小
        sum_min = np.inf
        s_v_rest = None
        for i in range(1, 50):
            s_real_one = 1000 * (1 - S) / i
            phi_s_i = np.where((1 - S) / i > 0.5, 0, 0.5 - (1 - S) / i)
            s_v_one = s_real_one * (1 + K * (1 - C) / i * phi_s_i)
            total = np.sum(s_v_one * i)
            if total < sum_min:
                sum_min = total
                s_v_rest = s_v_one * i
        s_v_total = s_v + s_v_rest
        return s_v / s_v_total


def draw_one(K=4):
    stake = np.linspace(0, 0.5, 100)
    contribution = np.linspace(0, 1, 100)
    S, C = np.meshgrid(stake, contribution)
    Z_adv = block_probability(S, C, K, strategy=2)
    Z_col = block_probability(S, C, K, strategy=3)

    fig = plt.figure(figsize=(12, 8))
    ax = fig.add_subplot(111, projection='3d')

    # 对抗策略：紫色半透明实面
    surf_adv = ax.plot_surface(
        S, C, Z_adv,
        cmap=cm.Purples,
        alpha=0.6,
        linewidth=0,
        antialiased=True,
        rstride=6, cstride=6
    )

    # 协作策略：灰色线框
    wire_col = ax.plot_wireframe(
        S, C, Z_col,
        rstride=6, cstride=6,
        linewidth=0.8,
        color='gray'
    )

    # —— 在 z=50% 处添加等高线 ——
    # 对 Equal‑Sharing 面
    ax.contour(
        S, C, Z_adv,
        levels=[0.5],  # 只画 z=0.5 的等高线
        zdir='z',
        offset=0.5,  # 将它放到 z=0.5 平面上
        colors=('purple',),
        linestyles='-',
        linewidths=2
    )
    # 对 Full‑Confrontation 面
    ax.contour(
        S, C, Z_col,
        levels=[0.5],
        zdir='z',
        offset=0.5,
        colors=('gray',),
        linestyles='--',
        linewidths=2
    )

    # —— 在 z 轴上标注 “50%” ——
    ax.text(
        0.5, 1, 0.5, '50%',
        color='black',
        fontsize=12,
        ha='left',
        va='bottom'
    )

    # 视角、标签
    ax.view_init(elev=35, azim=135)
    # ax.set_proj_type('ortho')
    # ax.set_box_aspect((1, 1, 1))
    ax.xaxis.set_pane_color((1.0, 1.0, 1.0, 1.0))
    ax.yaxis.set_pane_color((1.0, 1.0, 1.0, 1.0))
    ax.zaxis.set_pane_color((1.0, 1.0, 1.0, 1.0))
    for axis in (ax.xaxis, ax.yaxis, ax.zaxis):
        axis._axinfo['grid']['color'] = (0, 0, 0, 0.2)
        axis._axinfo['grid']['linewidth'] = 0.5  # 可选：线宽调小一些
    ax.set_xlabel('Real Stake Share', fontsize=16, labelpad=10)
    ax.set_ylabel('Network Contribution', fontsize=16, labelpad=10)
    ax.set_zlabel('Virtual Stake Share', fontsize=16, labelpad=10)

    ax.tick_params(axis='x', labelsize=12)
    ax.tick_params(axis='y', labelsize=12)
    ax.tick_params(axis='z', labelsize=12)

    # 百分比刻度
    ax.set_xticks([0, 0.25, 0.5])
    ax.set_yticks([0, 0.5, 1.0])
    ax.set_zticks([0, 0.25, 0.5])
    fmt = matplotlib.ticker.PercentFormatter(1.0)
    for ax_i in (ax.xaxis, ax.yaxis, ax.zaxis):
        ax_i.set_major_formatter(fmt)
    ax.tick_params(labelsize=10)

    # 让百分号标签更透明
    for lbl in ax.get_xticklabels():
        lbl.set_alpha(0.5)
    for lbl in ax.get_yticklabels():
        lbl.set_alpha(0.5)
    for lbl in ax.get_zticklabels():
        lbl.set_alpha(0.5)

    # 简洁图例内嵌
    legend_elems = [
        Patch(edgecolor='gray', facecolor='none', label='Equal‑Sharing'),
        Patch(edgecolor='purple', facecolor='none', label='Full-Confrontation'),
    ]
    ax.legend(handles=legend_elems,
              loc='upper left',
              fontsize=12,
              frameon=False)

    plt.tight_layout()
    plt.show()


def draw_one_small(K=4):
    stake = np.linspace(0, 0.5, 100)
    contribution = np.linspace(0, 1, 100)
    S, C = np.meshgrid(stake, contribution)
    Z_adv = block_probability(S, C, K, strategy=2)
    Z_col = block_probability(S, C, K, strategy=3)

    fig = plt.figure(figsize=(5, 4))  # 论文单栏宽度大约 5×4 inches
    ax = fig.add_subplot(111, projection='3d')

    # Adv 策略：半透明紫色面，稀疏网格
    ax.plot_surface(
        S, C, Z_adv,
        cmap=cm.Purples,
        alpha=0.5,
        rstride=8, cstride=8,
        linewidth=0.8,
        edgecolor='purple',
        antialiased=True,
        shade=False
    )

    # Col 策略：半透明灰色面，稀疏虚线
    ax.plot_surface(
        S, C, Z_col,
        cmap=cm.Greys,
        alpha=0.5,
        rstride=8, cstride=8,
        linewidth=0.8,
        edgecolor='gray',
        antialiased=True,
        linestyles='--',
        shade=False
    )

    # 视角
    ax.view_init(elev=30, azim=120)

    # 去掉次级网格，仅保留坐标轴底面
    ax.grid(False)
    for axis in (ax.xaxis, ax.yaxis, ax.zaxis):
        axis.set_pane_color((1, 1, 1, 1))
        axis._axinfo['grid']['color'] = (0, 0, 0, 0)

    # 精简刻度：三个点
    ax.set_xticks([0, 0.25, 0.5])
    ax.set_yticks([0, 0.5, 1.0])
    ax.set_zticks([0, 0.25, 0.5])
    fmt = matplotlib.ticker.PercentFormatter(1.0)
    ax.xaxis.set_major_formatter(fmt)
    ax.yaxis.set_major_formatter(fmt)
    ax.zaxis.set_major_formatter(fmt)
    ax.tick_params(labelsize=10)

    # 轴标签
    ax.set_xlabel('Real Stake', fontsize=12, labelpad=4)
    ax.set_ylabel('Network Contrib.', fontsize=12, labelpad=4)
    ax.set_zlabel('Virtual Stake', fontsize=12, labelpad=4)

    # 简洁图例内嵌
    legend_elems = [
        Patch(edgecolor='purple', facecolor='none', label='Other Nodes Equal‑Sharing'),
        Patch(edgecolor='gray', facecolor='none', label='Other Nodes Collusive‑Maximization')
    ]
    ax.legend(handles=legend_elems,
              loc='upper left',
              fontsize=10,
              frameon=False)

    plt.tight_layout()
    plt.show()


def draw_one_wire(K=4):
    stake = np.linspace(0, 0.5, 100)
    contribution = np.linspace(0, 1, 100)
    S, C = np.meshgrid(stake, contribution)
    Z_eq = block_probability(S, C, K, strategy=2)
    Z_col = block_probability(S, C, K, strategy=3)

    fig = plt.figure(figsize=(5, 4))  # 单栏宽度小图
    ax = fig.add_subplot(111, projection='3d')

    # 主网格辅助线（淡化）
    for xi in [0.0, 0.25, 0.5]:
        ax.plot_wireframe(
            np.full((2, 2), xi),
            np.array([[0, 1], [0, 1]]),
            np.array([[0, 0], [1, 1]]),
            color='lightgray', linestyle=':', linewidth=0.5, alpha=0.2
        )
    for yi in [0.0, 0.5, 1.0]:
        ax.plot_wireframe(
            np.array([[0, 0.5], [0, 0.5]]),
            np.full((2, 2), yi),
            np.array([[0, 0], [1, 1]]),
            color='lightgray', linestyle=':', linewidth=0.5, alpha=0.2
        )

    # Others’ Equal‑Sharing：黑色实线
    ax.plot_wireframe(
        S, C, Z_eq,
        rstride=10, cstride=10,
        color='black', linewidth=1.2, label='Others’ Equal‑Sharing'
    )
    # Others’ Collusive‑Maximization：灰色虚线
    ax.plot_wireframe(
        S, C, Z_col,
        rstride=10, cstride=10,
        color='gray', linewidth=1.2, linestyle='--', label='Others’ Collusive‑Maximization'
    )

    # 视角
    ax.view_init(elev=30, azim=120)

    # 坐标背景 & 网格
    ax.grid(False)
    for axis in (ax.xaxis, ax.yaxis, ax.zaxis):
        axis.set_pane_color((1, 1, 1, 1))
        axis._axinfo['grid']['color'] = (0, 0, 0, 0)

    # 刻度 & 百分比
    ax.set_xticks([0, 0.25, 0.5])
    ax.set_yticks([0, 0.5, 1.0])
    ax.set_zticks([0, 0.25, 0.5])
    fmt = matplotlib.ticker.PercentFormatter(1.0)
    for ax_i in (ax.xaxis, ax.yaxis, ax.zaxis):
        ax_i.set_major_formatter(fmt)
    ax.tick_params(labelsize=10)

    # 轴标签
    ax.set_xlabel('Real Stake', fontsize=12, labelpad=4)
    ax.set_ylabel('Network Contrib.', fontsize=12, labelpad=4)
    ax.set_zlabel('Virtual Stake', fontsize=12, labelpad=4)

    # 图例内嵌
    ax.legend(loc='upper left', fontsize=10, frameon=False)

    plt.tight_layout()
    plt.show()


if __name__ == '__main__':
    draw_one(8)
