import os
import numpy as np
import matplotlib.pyplot as plt
import matplotlib as mpl

# 设置matplotlib的样式
mpl.rcParams['font.sans-serif'] = ['DejaVu Sans']
mpl.rcParams['axes.unicode_minus'] = False

def generate_block_production_data():
    """
    生成模拟的不同掉线概率下各共识算法的出块成功率数据
    掉线概率范围: 0% - 50%（每个节点在每个时刻有X%的概率掉线）
    """
    offline_rates = np.array([0, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50])
    
    pog_success_rates = np.array([1, 142/(1+142), 162/(2+162), 160/(3+160), 156/(6+156),154/(8+154), 150/(10+150), 140/(10+140), 141/(11+141),150/(14+150) , 144/(16+144)]) * 100
    
    pos_success_rates = np.array([1, 155/(2+155),155/(5+155), 156/(10+156), 148/(14+148), 147/(18+147),148/(24+148), 120/(27+128),102/(50+102), 91/(63+91), 70/(85+76)]) * 100
    
    
    return offline_rates, pog_success_rates, pos_success_rates

def create_block_production_rate_figure():
    """
    绘制不同掉线率下的出块成功率对比图
    """
    # 获取数据
    offline_rates, pog_rates, pos_rates = generate_block_production_data()
    
    # 创建图表
    fig, ax = plt.subplots(figsize=(10, 8))
    
    # 绘制三条曲线
    ax.plot(offline_rates, pog_rates, 
            color='#1f77b4', linewidth=2.5, linestyle='-', 
            marker='o', markersize=8, label='PoG', zorder=3)
    
    ax.plot(offline_rates, pos_rates, 
            color='#2ca02c', linewidth=2.5, linestyle='--', 
            marker='s', markersize=8, label='PoS', zorder=3)

    
    # 设置网格
    ax.grid(True, linestyle='--', alpha=0.5, linewidth=0.7, zorder=1)
    ax.set_axisbelow(True)
    
    # 设置轴样式
    ax.spines['top'].set_linewidth(1.5)
    ax.spines['bottom'].set_linewidth(1.5)
    ax.spines['left'].set_linewidth(1.5)
    ax.spines['right'].set_linewidth(1.5)
    
    # 设置标签字体大小
    ax.set_xlabel('Offline probability per node per epoch (%)', fontsize=22, fontweight='normal')
    ax.set_ylabel('Block production success rate (%)', fontsize=22, fontweight='normal')
    # ax.set_title('Impact of offline probability on block production', fontsize=22, fontweight='normal', pad=20)
    
    # 设置坐标轴刻度字体大小
    ax.tick_params(axis='x', labelsize=18)
    ax.tick_params(axis='y', labelsize=18)
    
    # 设置坐标轴范围
    ax.set_xlim(-2, 52)
    ax.set_ylim(30, 100)
    
    # 设置图例
    ax.legend(fontsize=26, loc='best', frameon=True, fancybox=False, edgecolor='black')
    
    plt.tight_layout()
    project_root = get_project_root()
    output_file = os.path.join(project_root, 'figures', 'block_production_rate.png')
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print("[1/1] Block production rate figure generated successfully!")
    print("      Figure saved as: block_production_rate.png")
    plt.show()

def get_project_root():
    """自动查找项目根目录，通过寻找 Cargo.toml 文件"""
    current_dir = os.path.dirname(os.path.abspath(__file__))
    while current_dir != os.path.dirname(current_dir):
        if os.path.exists(os.path.join(current_dir, 'Cargo.toml')):
            return current_dir
        current_dir = os.path.dirname(current_dir)
    return current_dir

if __name__ == '__main__':
    print("\n========== Block Production Rate Analysis ==========\n")
    create_block_production_rate_figure()
    print("\n====================================================\n")
