pull_axum_notes branch_name="chpt10-axum/securing-api":
  git checkout {{branch_name}} -- notebooks

# to markdown
tmd notebook:
  jupyter nbconvert --to markdown {{notebook}}
