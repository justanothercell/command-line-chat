# register
`curl -X POST "http://localhost:8000/register" -H "Content-Type: application/json" -d "{ \"user_id\": 1 }"`
# unregister
`curl -X DELETE "http://localhost:8000/register/<uuid>"`
# post topic
`curl -X POST "http://localhost:8000/publish" -H "Content-Type: application/json" -d "{\"user_id\": 1, \"topic\": \"cats\", \"message\": \"are awesome\"}"`