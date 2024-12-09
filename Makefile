all: ui backend
	cp -r frontend/dist backend/dist

ui: frontend/dist

frontend/dist: 
	cd frontend; \
	npm run build

backend:
	cd backend; \
	go run cmd/main.go

clean:
	go clean
	rm -rf backend/dist frontend/dist

