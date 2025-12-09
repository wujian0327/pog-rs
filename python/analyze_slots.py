import os
import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
from pathlib import Path

# 设置中文字体
plt.rcParams['font.sans-serif'] = ['SimHei', 'DejaVu Sans']
plt.rcParams['axes.unicode_minus'] = False


def get_project_root():
    """自动查找项目根目录，通过寻找 Cargo.toml 文件"""
    current_dir = os.path.dirname(os.path.abspath(__file__))
    while current_dir != os.path.dirname(current_dir):
        if os.path.exists(os.path.join(current_dir, 'Cargo.toml')):
            return current_dir
        current_dir = os.path.dirname(current_dir)
    return current_dir


def read_metrics_csv(consensus_type):
    """读取指定共识算法的 CSV 文件"""
    project_root = get_project_root()
    csv_file = os.path.join(project_root, f'metrics_slots_{consensus_type}.csv')
    
    if not os.path.exists(csv_file):
        print(f"警告: 找不到文件 {csv_file}")
        return None
    
    try:
        df = pd.read_csv(csv_file)
        print(f"成功读取 {consensus_type} 的数据: {len(df)} 条记录")
        return df
    except Exception as e:
        print(f"读取 {csv_file} 出错: {e}")
        return None




def create_gini_line_figure(dataframes_dict):
    """创建 Gini 系数折线图"""
    if not dataframes_dict:
        print("没有有效的数据")
        return
    
    fig, ax = plt.subplots(figsize=(14, 7))
    
    colors = {'pog': '#FF6B6B', 'pos': '#4ECDC4', 'pow': '#45B7D1'}
    
    # 绘制每种共识的 Gini 系数折线
    for ct, df in dataframes_dict.items():
        if df is not None and len(df) > 0:
            # 使用 index 作为横坐标，gini_coefficient 作为 Gini 系数
            ax.plot(df.index, df['gini_coefficient'], label=ct.upper(), 
                   color=colors.get(ct, '#000000'), linewidth=2.5, alpha=0.85, marker='o', markersize=4)
    
    ax.set_xlabel('Index', fontsize=12, fontweight='bold')
    ax.set_ylabel('Gini 系数', fontsize=12, fontweight='bold')
    ax.set_title('Gini 系数变化趋势', fontsize=14, fontweight='bold')
    ax.legend(fontsize=12, loc='best')
    ax.grid(True, alpha=0.3, linestyle='--')
    ax.set_ylim(0, 0.8)
    
    plt.tight_layout()
    
    # 保存图表
    project_root = get_project_root()
    output_file = os.path.join(project_root, 'figures', 'gini_coefficient.png')
    os.makedirs(os.path.dirname(output_file), exist_ok=True)
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"Gini 系数图表已保存到: {output_file}")
    plt.show()


def create_tps_line_figure(dataframes_dict):
    """创建 TPS (吞吐量) 折线图"""
    if not dataframes_dict:
        print("没有有效的数据")
        return
    
    fig, ax = plt.subplots(figsize=(14, 7))
    
    colors = {'pog': '#FF6B6B', 'pos': '#4ECDC4', 'pow': '#45B7D1'}
    
    # 绘制每种共识的 TPS 折线
    for ct, df in dataframes_dict.items():
        if df is not None and len(df) > 0:
            # 计算累计平均 TPS
            cumulative_mean_tps = df['throughput'].expanding().mean()
            
            # 使用 index 作为横坐标，累计平均 throughput 作为 TPS
            ax.plot(df.index, cumulative_mean_tps, label=ct.upper(), 
                   color=colors.get(ct, '#000000'), linewidth=2.5, alpha=0.85)
    
    ax.set_xlabel('Index', fontsize=12, fontweight='bold')
    ax.set_ylabel('吞吐量 (tx/s)', fontsize=12, fontweight='bold')
    ax.set_title('TPS (吞吐量) 累计平均变化趋势', fontsize=14, fontweight='bold')
    ax.legend(fontsize=12, loc='best')
    ax.grid(True, alpha=0.3, linestyle='--')
    
    plt.tight_layout()
    
    # 保存图表
    project_root = get_project_root()
    output_file = os.path.join(project_root, 'figures', 'tps_throughput.png')
    os.makedirs(os.path.dirname(output_file), exist_ok=True)
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"TPS 图表已保存到: {output_file}")
    plt.show()


def create_path_length_line_figure(dataframes_dict):
    """创建交易平均路径长度折线图"""
    if not dataframes_dict:
        print("没有有效的数据")
        return
    
    fig, ax = plt.subplots(figsize=(14, 7))
    
    colors = {'pog': '#FF6B6B', 'pos': '#4ECDC4', 'pow': '#45B7D1'}
    
    # 绘制每种共识的平均路径长度折线
    for ct, df in dataframes_dict.items():
        if df is not None and len(df) > 0:
            # 计算累计平均路径长度
            cumulative_mean_path = df['avg_path_length'].expanding().mean()
            
            # 使用 index 作为横坐标，累计平均 avg_path_length 作为平均路径
            ax.plot(df.index, cumulative_mean_path, label=ct.upper(), 
                   color=colors.get(ct, '#000000'), linewidth=2.5, alpha=0.85)
    
    ax.set_xlabel('Index', fontsize=12, fontweight='bold')
    ax.set_ylabel('平均路径长度', fontsize=12, fontweight='bold')
    ax.set_title('交易平均路径长度累计平均变化趋势', fontsize=14, fontweight='bold')
    ax.legend(fontsize=12, loc='best')
    ax.grid(True, alpha=0.3, linestyle='--')
    
    plt.tight_layout()
    
    # 保存图表
    project_root = get_project_root()
    output_file = os.path.join(project_root, 'figures', 'path_length.png')
    os.makedirs(os.path.dirname(output_file), exist_ok=True)
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"平均路径长度图表已保存到: {output_file}")
    plt.show()


def create_trend_figures(dataframes_dict):
    """创建趋势图表（随时间变化）- 已弃用"""
    pass


def print_summary(metrics_dict):
    """打印统计摘要"""
    print("\n" + "="*70)
    print("共识算法性能统计摘要")
    print("="*70)
    
    for ct, metrics in metrics_dict.items():
        if metrics:
            print(f"\n【{ct.upper()} 共识】")
            print(f"  Gini 系数:     {metrics['avg_gini']:.6f} (范围: {metrics['min_gini']:.6f} ~ {metrics['max_gini']:.6f})")
            print(f"  吞吐量:        {metrics['avg_throughput']:.2f} tx/s (范围: {metrics['min_throughput']:.2f} ~ {metrics['max_throughput']:.2f})")
            print(f"  平均路径长度:  {metrics['avg_path_length']:.2f} (范围: {metrics['min_path_length']:.2f} ~ {metrics['max_path_length']:.2f})")
            print(f"  平均交易延迟:  {metrics['avg_tx_delay']:.2f} ms (范围: {metrics['min_tx_delay']:.2f} ~ {metrics['max_tx_delay']:.2f})")
    
    print("\n" + "="*70)


if __name__ == '__main__':
    import sys
    
    print("开始分析共识算法性能指标...\n")
    
    # 读取三种共识的数据
    consensus_types = ['pog', 'pos', 'pow']
    dataframes_dict = {}
    
    for ct in consensus_types:
        df = read_metrics_csv(ct)
        if df is not None:
            dataframes_dict[ct] = df
    
    # 是否显示图表（可通过命令行参数 --show 控制）
    show_plots = '--show' in sys.argv
    
    # 创建 Gini 系数折线图
    if dataframes_dict:
        print("\n生成 Gini 系数图表...")
        create_gini_line_figure(dataframes_dict)
        if not show_plots:
            plt.close('all')
    
    # 创建 TPS 折线图
    if dataframes_dict:
        print("\n生成 TPS 图表...")
        create_tps_line_figure(dataframes_dict)
        if not show_plots:
            plt.close('all')
    
    # 创建平均路径长度折线图
    if dataframes_dict:
        print("\n生成交易平均路径长度图表...")
        create_path_length_line_figure(dataframes_dict)
        if not show_plots:
            plt.close('all')
    
    print("\n分析完成！")
    print("图表已保存到 figures/ 目录:")
    print("  - gini_coefficient.png (Gini 系数)")
    print("  - tps_throughput.png (TPS 吞吐量)")
    print("  - path_length.png (交易平均路径长度)")
    if not show_plots:
        print("如要显示图表，请运行: python python/analyze_slots.py --show")
