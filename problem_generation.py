import random

# Generates a problem files for knap-sack problem
# according to ECMM409-Coursework constraints
def generate_bags(filename, N):
    with open(filename, 'w') as file:
        file.write("security van capacity: 295\n")
        for i in range(1, N + 1):
            weight = round(random.uniform(1, 10), 1)
            cost = random.randint(10, 100)
            file.write(f" bag {i}:\n")
            file.write(f"  weight: {weight}\n")
            file.write(f"  value: {cost}\n")

# N for number of bags
N = 500
generate_bags(f"src/problem{N}.txt", N)
