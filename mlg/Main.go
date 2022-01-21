package main

import (
	"bufio"
	"crypto/md5"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"io/ioutil"
	"net/http"
	"os"
	"os/exec"
	"path"
	"strings"
)

const MATHLINGUA_VERSION = "0.15.1"

const DOT_MLG_NAME = ".mlg"
const MATHLINGUA_JAR_NAME = "mathlingua.jar"
const HASH_FILE_NAME = "hash"

func getDotMlgPath() (string, error) {
	cwd, err := os.Getwd()
	if err != nil {
		return "", err
	}
	return path.Join(cwd, DOT_MLG_NAME), nil
}

func getPathWithinDotMlgDir(name string) (string, error) {
	dotMlgDir, err := getDotMlgPath()
	if err != nil {
		return "", err
	}
	return path.Join(dotMlgDir, name), nil
}

func getMathLinguaJarPath() (string, error) {
	return getPathWithinDotMlgDir(MATHLINGUA_JAR_NAME)
}

func runMathLingua(args []string) (int, error) {
	jarFile, err := getMathLinguaJarPath()
	if err != nil {
		return -1, err
	}

	cmdArgs := []string{"-jar", jarFile}
	cmdArgs = append(cmdArgs, args...)
	return runCommand("java", cmdArgs)
}

func runCommand(program string, args []string) (int, error) {
	cmd := exec.Command(program, args...)

	stdout, err := cmd.StdoutPipe()
	if err != nil {
		return 1, err
	}

	stderr, err := cmd.StderrPipe()
	if err != nil {
		return 1, err
	}

	err = cmd.Start()
	if err != nil {
		return 1, err
	}

	combined := io.MultiReader(stdout, stderr)
	scanner := bufio.NewScanner(combined)
	for scanner.Scan() {
		fmt.Println(scanner.Text())
	}

	err = cmd.Wait()
	if err != nil {
		return 1, err
	}

	return cmd.ProcessState.ExitCode(), nil
}

func getLatestVersion() (string, error) {
	allVersions, err := getAllVersions()
	if err != nil {
		return "", err
	}

	if len(allVersions) == 0 {
		return "", errors.New("No releases exist")
	}

	return allVersions[0], nil
}

// version is either "latest" or of the form "x.x.x"
// the string returned is the url for the mathlingua.jar file
// for the given version
func getRelease(version string) (string, error) {
	versionString := ""
	if version == "latest" {
		latestVersion, err := getLatestVersion()
		if err != nil {
			return "", err
		}
		versionString = "-" + latestVersion
	} else {
		versionString = "-" + version
	}

	allUrls, err := getAllUrls()
	if err != nil {
		return "", err
	}

	jarUrl := ""
	for _, url := range allUrls {
		if strings.Contains(url, "mathlingua") &&
			strings.Contains(url, versionString) &&
			strings.HasSuffix(url, ".jar") {
			jarUrl = url
			break
		}
	}

	return jarUrl, nil
}

func getAllVersions() ([]string, error) {
	allUrls, err := getAllUrls()
	if err != nil {
		return nil, err
	}

	result := []string{}
	for _, url := range allUrls {
		withoutPrefix := strings.ReplaceAll(url,
			"https://github.com/DominicKramer/mathlingua/releases/download/v", "")

		index := strings.Index(withoutPrefix, "/")
		version := ""
		if index < 0 {
			version = withoutPrefix
		} else {
			version = withoutPrefix[:index]
		}

		if len(result) == 0 || result[len(result)-1] != version {
			result = append(result, version)
		}
	}

	return result, nil
}

func getAllUrls() ([]string, error) {
	resp, err := http.Get(
		"https://api.github.com/repos/DominicKramer/mathlingua/releases")
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	data, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}

	var jsonData []map[string]interface{}
	json.Unmarshal(data, &jsonData)

	urls := []string{}
	for _, item := range jsonData {
		castErr := errors.New("Could not determine download url")

		assets, ok := item["assets"].([]interface{})
		if !ok {
			return nil, castErr
		}

		if len(assets) == 0 {
			return nil, castErr
		}

		for _, asset := range assets {
			resultMap, ok := asset.(map[string]interface{})
			if !ok {
				return nil, castErr
			}

			browserDownloadUrl := resultMap["browser_download_url"].(string)
			if !ok {
				return nil, castErr
			}

			urls = append(urls, browserDownloadUrl)
		}
	}

	return urls, nil
}

func downloadUrl(url string, to string) error {
	outPath := to + ".part"
	outputFile, err := os.Create(outPath)
	if err != nil {
		return err
	}
	defer outputFile.Close()

	resp, err := http.Get(url)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	_, err = io.Copy(outputFile, resp.Body)
	if err != nil {
		return err
	}

	return os.Rename(outPath, to)
}

func ensureDirExists(dir string) error {
	_, err := os.Stat(dir)
	if os.IsNotExist(err) {
		err := os.MkdirAll(dir, 0755)
		if err != nil {
			return err
		}
	}
	return nil
}

func ensureMathLinguaJarExists(version string, isUpdating bool) (bool, error) {
	dotMlgDir, err := getDotMlgPath()
	if err != nil {
		return false, err
	}

	err = ensureDirExists(dotMlgDir)
	if err != nil {
		return false, err
	}

	jarfile, err := getMathLinguaJarPath()
	if err != nil {
		return false, err
	}

	_, err = os.Stat(jarfile)
	if os.IsNotExist(err) {
		if !isUpdating {
			fmt.Println("Initial run of mlg detected...")
			fmt.Println()
		}

		if version == "latest" {
			fmt.Println("Downloading the latest version of MathLingua core...")
		} else {
			fmt.Println("Downloading version " + version + " of MathLingua core...")
		}

		jarUrl, err := getRelease(version)
		if err != nil {
			return false, err
		}

		if jarUrl == "" {
			if version == "latest" {
				return false, errors.New("ERROR: Unable to find the latest version of MathLingua")
			} else {
				return false, errors.New("ERROR: Unable to find version " + version + " of MathLingua")
			}
		}

		// download the mathlingua.jar file
		err = downloadUrl(jarUrl, jarfile)
		if err != nil {
			if version == "latest" {
				return false, errors.New("ERROR: An error occurred downloading the latest version of MathLingua")
			} else {
				return false, errors.New("ERROR: An error occurred downloading version " + version + " of MathLingua")
			}
		}

		return true, nil
	}

	return true, nil
}

func copy(src, dest string) error {
	from, err := os.Open(src)
	if err != nil {
		return err
	}
	defer from.Close()

	to, err := os.Create(dest)
	if err != nil {
		return err
	}
	defer to.Close()

	_, err = io.Copy(to, from)
	return err
}

func storeHashValue(path, hash string) error {
	file, err := os.Create(path)
	if err != nil {
		return err
	}

	defer file.Close()

	_, err = file.WriteString(hash)
	if err != nil {
		return err
	}

	return nil
}

func loadHashValue(path string) (string, error) {
	if _, err := os.Stat(path); errors.Is(err, os.ErrNotExist) {
		return "", nil
	}

	content, err := ioutil.ReadFile(path)
	if err != nil {
		return "", err
	}

	return string(content), nil
}

func getFileHash(path string) (string, error) {
	file, err := os.Open(path)
	if err != nil {
		return "", err
	}

	defer file.Close()

	hash := md5.New()
	_, err = io.Copy(hash, file)

	if err != nil {
		return "", err
	}

	result := fmt.Sprintf("%x", hash.Sum(nil))
	return result, nil
}

func deleteFile(path string) error {
	err := os.Remove(path)
	if err != nil {
		if os.IsNotExist(err) {
			return nil
		} else {
			return err
		}
	}

	return nil
}

func main() {
	programPath := os.Args[0]

	dotMlgDir, err := getDotMlgPath()
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}

	err = ensureDirExists(dotMlgDir)
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}

	mathlinguaJarFile, err := getMathLinguaJarPath()
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}

	programHash, err := getFileHash(programPath)
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}

	hashFile, err := getPathWithinDotMlgDir(HASH_FILE_NAME)
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}

	storedHash, err := loadHashValue(hashFile)
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}

	if programHash != storedHash {
		// This occurs if the user has downloaded a new mlg binary file.
		// Thus, in this case, delete the hash and mathlingua.jar files
		// so the jar file for the new mlg file is downloaded.

		err := deleteFile(hashFile)
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}

		err = deleteFile(mathlinguaJarFile)
		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}
	}

	args := os.Args[1:]

	// when mlg is initially run, it will download version MATHLINGUA_VERSION
	version := MATHLINGUA_VERSION
	isUpdating := false
	if len(args) >= 1 {
		if args[0] == "update" {
			isUpdating = true

			// remove the DOT_MLG_NAME directory so that the specified
			// version or the latest version is downloaded
			os.RemoveAll(DOT_MLG_NAME)

			if len(args) >= 2 {
				// use the version given if specified
				if strings.HasPrefix(args[1], "v") {
					version = args[1][1:]
				} else {
					version = args[1]
				}
			} else {
				// otherwise if an explicit version isn't given, update to the latest
				version = "latest"
			}
		} else if args[0] == "versions" {
			versions, err := getAllVersions()
			if err != nil {
				fmt.Println(err)
				os.Exit(1)
			}

			for _, version := range versions {
				fmt.Println(version)
			}

			os.Exit(0)
		}
	}

	runJarFile, err := ensureMathLinguaJarExists(version, isUpdating)
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}

	// Store the hash of the mlg file so that on the next run of mlg the
	// mathlingua.jar file doesn't get downloaded again.
	err = storeHashValue(hashFile, programHash)
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}

	if !runJarFile {
		os.Exit(0)
	}

	programArgs := args
	if isUpdating {
		programArgs = []string{"version"}
	}

	exitCode, err := runMathLingua(programArgs)
	if err != nil {
		fmt.Println(err)
	}
	os.Exit(exitCode)
}
