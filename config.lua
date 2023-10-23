-- Utility function to list all files in a directory
local function scandir(directory)
	local i, t, popen = 0, {}, io.popen
	local pfile = popen('ls "' .. directory .. '"')

	if pfile == nil then
		return {}
	end

	for filename in pfile:lines() do
		i = i + 1
		t[i] = filename
	end
	pfile:close()
	return t
end

-- Convert the selected path into a session name used by tmux
function session_name(path)
	if path.path == os.getenv("HOME") then
		return "home"
	end

	return path.filename:gsub("%.", "_"):gsub(",", "_"):gsub(" ", "_")
end

-- List all paths that the user can select from
function list_projects()
	local root = os.getenv("HOME") .. "/Projects"
	return scandir(root)
end

-- Define which windows the session should contain
function get_layout(path)
	local session_name = path.session_name

	if session_name == "home" then
		return { "zsh" }
	end

	return {
		"zsh -c nvim",
		"zsh"
	}
end
