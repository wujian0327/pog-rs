import numpy as np
import pandas as pd
import matplotlib.pyplot as plt
import os

# 设置科研风格
plt.style.use('seaborn-v0_8-whitegrid')
plt.rcParams['font.sans-serif'] = ['SimHei', 'DejaVu Sans']
plt.rcParams['axes.unicode_minus'] = False
plt.rcParams['figure.dpi'] = 100
plt.rcParams['savefig.dpi'] = 300
plt.rcParams['font.size'] = 10
plt.rcParams['axes.labelsize'] = 11
plt.rcParams['axes.titlesize'] = 12
plt.rcParams['xtick.labelsize'] = 9
plt.rcParams['ytick.labelsize'] = 9
plt.rcParams['legend.fontsize'] = 10
plt.rcParams['lines.linewidth'] = 2.5
plt.rcParams['axes.grid'] = True
plt.rcParams['grid.alpha'] = 0.4

def pog_logarithmic_saturation(raw_score, k_sat=1.0, k_base=1.0):
    """
    PoG 的对数饱和函数 (对应 src/consensus/pog.rs 中的 cal_slot_contribution)
    C_slot(n,t) = K_sat * log(1 + raw_score / K_base)
    """
    return k_sat * np.log(1 + raw_score / k_base)

def calculate_virtual_stake(normalized_contribution, normalized_stake, omega=1.0):
    """
    计算虚拟权益 (对应 src/consensus/pog.rs 中的 cal_virtual_stake)
    S_v = omega * hat_C + (1 - omega) * hat_S
    """
    return omega * normalized_contribution + (1 - omega) * normalized_stake

def simulate_saturation_defense():
    # PoG 参数 (参考 src/consensus/pog.rs)
    n_honest = 99
    base_raw_score = 100.0
    k_sat = 1.0
    k_base = 1.0
    omega = 1.0 # 纯 PoG 模式 (omega=1.0)
    
    # 经济参数
    block_reward = 1.0 # 区块奖励
    fee_rate = 0.00001 # 手续费率 (相对于 Block Reward)
    
    # 假设所有节点权益相同 (Stake)
    # 诚实节点 Stake = 1, 攻击者 Stake = 1
    # Normalized Stake (hat_S)
    total_stake = n_honest * 1.0 + 1.0
    hat_s_honest = 1.0 / total_stake
    hat_s_attacker = 1.0 / total_stake
    
    multipliers = np.linspace(1, 50, 100) # 1x 到 50x 攻击倍率
    
    results = []
    
    for m in multipliers:
        # 1. 计算 Raw Scores
        raw_h = base_raw_score
        raw_a = base_raw_score * m
        
        # 2. 应用 PoG 饱和函数
        sat_h = pog_logarithmic_saturation(raw_h, k_sat, k_base)
        sat_a = pog_logarithmic_saturation(raw_a, k_sat, k_base)
        
        # 3. 计算 Normalized Contribution (hat_C)
        total_sat = n_honest * sat_h + sat_a
        hat_c_honest = sat_h / total_sat
        hat_c_attacker = sat_a / total_sat
        
        # 4. 计算 Virtual Stake (S_v) - PoG 最终选择概率
        sv_honest = calculate_virtual_stake(hat_c_honest, hat_s_honest, omega)
        sv_attacker = calculate_virtual_stake(hat_c_attacker, hat_s_attacker, omega)
        
        # 归一化 Virtual Stake (作为最终概率)
        total_sv = n_honest * sv_honest + sv_attacker
        prob_pog = sv_attacker / total_sv
        
        # --- 经济分析 ---
        # 成本: 交易量 * 费率
        cost_attacker = raw_a * fee_rate
        # 收益: 概率 * 区块奖励
        revenue_attacker = prob_pog * block_reward
        # 净收益
        net_profit = revenue_attacker - cost_attacker
        
        results.append({
            'Multiplier': m,
            'Share_Log': prob_pog,
            'Cost': cost_attacker,
            'Revenue': revenue_attacker,
            'Net_Profit': net_profit
        })
        
    return pd.DataFrame(results)

def plot_spam_saturation():
    df = simulate_saturation_defense()
    
    fig, ax = plt.subplots(figsize=(10, 7))
    
    # 颜色定义
    color_revenue = '#2ca02c'  # 绿色 (收益)
    color_cost = '#d62728'     # 红色 (成本)
    
    # 绘制曲线
    ax.plot(df['Multiplier'], df['Revenue'], '-', color=color_revenue, linewidth=4, label='Total Revenue (Block Reward)')
    ax.plot(df['Multiplier'], df['Cost'], '--', color=color_cost, linewidth=3, label='Total Cost (Transaction Fees)')
    
    # 填充区域
    ax.fill_between(df['Multiplier'], df['Revenue'], df['Cost'], 
                    where=(df['Revenue'] < df['Cost']), 
                    interpolate=True, color='red', alpha=0.1, label='Net Loss Area')
    
    # 装饰
    ax.set_xlabel('Spam Attack Intensity', fontsize=22, fontweight='bold')
    ax.set_ylabel('Economic Value (Tokens)', fontsize=22, fontweight='bold')
    ax.set_title('Spam Saturation Defense', fontsize=22, fontweight='bold', pad=20)
    
    ax.tick_params(axis='both', labelsize=18)
    
    # 强化边框
    for spine in ax.spines.values():
        spine.set_linewidth(1.5)
        spine.set_color('black')
        
    # 寻找盈亏平衡点
    # 找到 Cost > Revenue 的第一个点
    idx_loss = np.where(df['Cost'] > df['Revenue'])[0]
    if len(idx_loss) > 0:
        first_loss_idx = idx_loss[0]
        row_loss = df.iloc[first_loss_idx]
        
        ax.annotate('Cost > Revenue', 
                    xy=(row_loss['Multiplier'], row_loss['Cost']), 
                    xytext=(row_loss['Multiplier'] + 5, row_loss['Cost'] - 0.005),
                    arrowprops=dict(facecolor='red', shrink=0.05),
                    fontsize=16, fontweight='bold', color='red',
                    bbox=dict(boxstyle="round,pad=0.3", fc="white", ec='red', alpha=0.9))


    ax.legend(fontsize=18, loc='best', frameon=True, fancybox=False, edgecolor='black', framealpha=0.95)
    ax.grid(True, alpha=0.5, linestyle='--', linewidth=0.7, color='gray')
    
    # 保存
    output_dir = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), 'figures')
    if not os.path.exists(output_dir):
        os.makedirs(output_dir)
        
    output_path = os.path.join(output_dir, 'spam_saturation_defense.png')
    plt.savefig(output_path, dpi=300, bbox_inches='tight')
    print(f"Figure saved to: {output_path}")

if __name__ == "__main__":
    try:
        plot_spam_saturation()
    except Exception as e:
        import traceback
        traceback.print_exc()
