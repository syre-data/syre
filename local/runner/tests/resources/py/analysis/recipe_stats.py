# include packages
import pandas as pd
import syre

# initialize database
db = syre.Database()

# get recipe container
recipe = db.root

# get noise statistics data
noise_stats = db.find_assets(type='noise-stats')

# create combined dataframe 
df = []
for stat in noise_stats:
	# read data for each batch
	tdf = pd.read_csv( 
		stat.file, 
		names = (stat.metadata['batch'],), 
		index_col = 0, 
		header = 0 
	)
	
	df.append(tdf)

df = pd.concat(df, axis = 1)

# compute recipe statistics
mean = df.loc['mean'].mean() 
std = df.loc['std'].pow(2).sum()/4 

stats = pd.DataFrame([mean, std], index = ('mean', 'std'))

# export recipe statistics
stats_path = db.add_asset(
    'recipe-stats.pkl',
	name='{} Statistics'.format(recipe.name),
	type='recipe-stats'
)

stats.to_pickle(stats_path)
