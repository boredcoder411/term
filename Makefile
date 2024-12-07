all: ui backend
	cp -r frontend/dist backend/dist

ui:
	cd frontend; \
		npm run build

backend:
	cd backend; \
		cargo build --release

clean:
	cd backend; \
		cargo clean
		rm -rf backend/dist
	cd frontend; \
		rm -rf dist
