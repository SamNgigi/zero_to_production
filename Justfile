pull_actix_notes branch_name="chpt10-actix/securing-api":
  git checkout {{branch_name}} -- notebooks

tnb markdown:
  jupytext --to notebook {{markdown}}

tmd notebook:
  jupyter nbconvert --to markdown {{notebook}}
