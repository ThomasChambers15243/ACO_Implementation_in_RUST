import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt

# Load each dataset
data_ant_num = pd.read_csv("csv/results_ant_num.csv")
data_p_rate = pd.read_csv("csv/results_p_rate.csv")
data_evaporation = pd.read_csv("csv/results_evaporation.csv")

# Plot 
plt.figure(figsize=(18, 8))

# Number of Ants
plt.subplot(1, 3, 1)
sns.lineplot(data=data_ant_num, x='Number_Of_Ants', y='Initial_fitness', marker='o', label='Initial Top Value')
sns.lineplot(data=data_ant_num, x='Number_Of_Ants', y='Top_Fitness', marker='o', label='Top Value')
plt.title('Effect of Ants Population')
plt.xlabel('Ants Population')
plt.ylabel('Max Path Value')
plt.legend()

# P Rate
plt.subplot(1, 3, 2)
sns.lineplot(data=data_p_rate, x='p_rate', y='Initial_fitness', marker='o', label='Initial Top Value')
sns.lineplot(data=data_p_rate, x='p_rate', y='Top_Fitness', marker='o', label='Top Value')
plt.title('Effect of Pheromone Rate')
plt.xlabel('Pheromone Rate')
plt.ylabel('Max Path Value')
plt.legend()

# Evaporation Rate
plt.subplot(1, 3, 3)
sns.lineplot(data=data_evaporation, x='Evaporation_Rate', y='Initial_fitness', marker='o', label='Initial Top Value')
sns.lineplot(data=data_evaporation, x='Evaporation_Rate', y='Top_Fitness', marker='o', label='Top Value')
plt.title('Effect of Evaporation Rate')
plt.xlabel('Evaporation Rate')
plt.ylabel('Max Path Value')
plt.legend()

plt.tight_layout()
plt.show()


# Load each dataset
data_ant_num = pd.read_csv("csv/results_ant_num.csv")
data_p_rate = pd.read_csv("csv/results_p_rate.csv")
data_evaporation = pd.read_csv("csv/results_evaporation.csv")

# Get mean values for each parameter type
avg_ant_num = data_ant_num.groupby('Parameter')['Top_Fitness'].mean().reset_index()
avg_ant_num['Parameter_Type'] = 'Number_Of_Ants'

avg_p_rate = data_p_rate.groupby('Parameter')['Top_Fitness'].mean().reset_index()
avg_p_rate['Parameter_Type'] = 'p_rate'

avg_evaporation = data_evaporation.groupby('Parameter')['Top_Fitness'].mean().reset_index()
avg_evaporation['Parameter_Type'] = 'Evaporation_Rate'

# Combine the averaged datasets
avg_combined = pd.concat([avg_ant_num, avg_p_rate, avg_evaporation], ignore_index=True)

# Values are hardcoded intune with the experiment configfuration in main.rs
xs = [
    "Ants: 2\nEvap: 0.1\nP_rate: .5",
    "Ants: 5\nEvap: 0.2\nP_rate: 1",
    "Ants: 10\nEvap: 0.3\nP_rate: 2",
    "Ants: 15\nEvap: 0.4\nP_rate: 3",
    "Ants: 20\nEvap: 0.5\nP_rate: 4",
    "Ants: 30\nEvap: 0.6\nP_rate: 5",
    "Ants: 50\nEvap: 0.7\nP_rate: 6",
    "Ants: 100\nEvap: 0.8\nP_rate: 7",
]

# Plot the average Top_Fitness for each parameter change on the same plot
plt.figure(figsize=(12, 6))
sns.lineplot(data=avg_combined, x='Parameter', y='Top_Fitness', hue='Parameter_Type', marker='o')
plt.title('Mean Top Value Across Parameter Changes')
plt.xlabel('Value per Perameter\nAll other values were set to their default values')
plt.ylabel('Mean Top Fitness')
plt.xticks(ticks=range(1, 9), labels=[f'{i}' for i in xs])
plt.grid()
plt.legend(title='Parameters Changed')
plt.show()