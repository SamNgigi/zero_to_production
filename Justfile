pull_axum_notes branch_name="chpt11-axum/fault-tolerant-workflows":
  git checkout {{branch_name}} -- notebooks

# to markdown
tmd notebook:
  jupyter nbconvert --to markdown {{notebook}}
