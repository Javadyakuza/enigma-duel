import React, { useEffect, useState } from "react";
import { Button, Typography } from "antd";
import "antd/dist/reset.css";
import { useNavigate, useLocation } from "react-router-dom";
import "./App.css"; // Make sure to import your CSS file
import {
  SearchOutlined,
  ArrowDownOutlined,
  ArrowUpOutlined,
  ArrowLeftOutlined,
  CheckOutlined,
  CloseOutlined,
} from "@ant-design/icons";
import { Col, InputNumber, Row, Slider, Space } from "antd";
import {
  connect_wallet,
  deposit,
  withdraw,
  fetch_question,
  getFormattedString,
  sleep,
} from "./helpers";
import { Buffer } from "buffer";
import process from "process";
import { ConnectionNotOpenError } from "web3";

const { Title } = Typography;
global.Buffer = Buffer;
global.process = process;
// Function to fetch data

const fetchGameData = async () => {
  // Simulate a data fetch. Replace with actual data fetching logic.
  const response = await new Promise((resolve) => {
    setTimeout(() => {
      resolve({ opponent: "some serious address", questions: "3" });
    }, 1000);
  });
  return response;
};
const fetchResults = async () => {
  // Simulate a data fetch. Replace with actual data fetching logic.
  const response = await new Promise((resolve) => {
    setTimeout(() => {
      resolve({
        user_res: [true, false, true],
        opponent_res: [false, false, true],
        status: "congrats you won 🎉",
      });
    }, 1000);
  });
  return response;
};
const Home = () => {
  const [walletData, setWalletData] = useState({ address: "", balance: "" });
  const [inputTokens, setInputTokens] = useState("1");
  const [findMatchDisability, setFindMatchDisability] = useState(true);

  const onChange = (newValue) => {
    setFindMatchDisability(find_match_dis());
    setInputTokens(newValue);
  };
  const fetchWalletData = async () => {
    let wd = await connect_wallet();
    setWalletData(wd);
    return wd;
  };
  const handleDeposit = async () => {
    await deposit(inputTokens);
  };
  const handleWithdraw = async () => {
    await withdraw(inputTokens);
  };

  const find_match_dis = () => {
    return walletData.balance >= inputTokens ? false : true;
  };
  useEffect(() => {
    const getData = async () => {
      const fetchedData = await fetchWalletData();
      fetchedData.address = (
        <Button
          type="primary"
          disabled={fetchedData.address == "" ? false : true}
          onClick={connect_wallet}
        >
          {fetchedData.address}
        </Button>
      );
      setWalletData(fetchedData);
    };

    getData();
  }, []);

  const navigate = useNavigate();
  const navigateToMatch = () => {
    navigate("/match");
  };
  return (
    <div className="main-container">
      <div className="main-inside-container">
        <div className="top-left">
          <Typography.Title style={{ color: "rgb(127, 25, 127)" }} level={5}>
            Address:{" "}
            <span style={{ color: "#d78c09" }}>{walletData.address}</span>
          </Typography.Title>
          <Typography.Title
            style={{ color: "rgb(127, 25, 127)", textAlign: "left" }}
            level={5}
          >
            Balance:{" "}
            <span style={{ color: "#d78c09" }}>{walletData.balance}</span>
          </Typography.Title>
        </div>
        <div className="center">
          <Title level={2} style={{ color: "rgb(255, 0, 255)" }}>
            {" "}
            Enigma Duel
          </Title>
        </div>
        <div className="bottom">
          <Button
            style={{ backgroundColor: "rgb(200, 78, 158)" }}
            icon={<SearchOutlined />}
            iconPosition={"start"}
            type="primary"
            onClick={navigateToMatch}
            disabled={findMatchDisability}
          >
            Find Match
          </Button>
          <Row>
            <Col span={12}>
              <Slider
                min={0}
                max={20}
                onChange={onChange}
                value={typeof inputTokens === "number" ? inputTokens : 0}
              />
            </Col>
            <Col span={4}>
              <InputNumber
                min={0}
                max={20}
                style={{
                  margin: "0 16px",
                }}
                value={inputTokens}
                onChange={onChange}
              />
            </Col>
          </Row>
          <div className="bottom-buttons">
            <Button
              // style={{ backgroundColor: "rgb(127, 25, 127)" }}
              icon={<ArrowDownOutlined />}
              iconPosition={"start"}
              type="primary"
              onClick={handleDeposit}
            >
              Deposit
            </Button>

            <Button
              style={{ backgroundColor: "rgb(60, 3144, 60)" }}
              icon={<ArrowUpOutlined />}
              iconPosition={"start"}
              type="primary"
              onClick={handleWithdraw}
            >
              Withdraw
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
};

const Match = () => {
  const [question, setQuestion] = useState({
    body: "",
    opt1: "",
    opt2: "",
    opt3: "",
    opt4: "",
    correct: "",
  });
  const [result, setResult] = useState([]);
  const fetchQuestion = async () => {
    if (gameData.questions == 0) {
      navigateToEndMatch();
    }
    setQuestion(await fetch_question());
  };

  const [gameData, setGameData] = useState({
    opponent: getFormattedString(
      "archtestop18t6uyv8797813bujb645qef2f234f234jsnbfv"
    ),
    questions: 3,
  });

  useEffect(() => {
    if (gameData.questions == 0) {
      navigateToEndMatch();
    }
    fetchQuestion();
  }, []);
  const navigate = useNavigate();

  const navigateToEndMatch = () => {
    let tmp = [false, true, true];
    console.log("sending this one: ", [...result, ...[false, true, true]]);
    navigate("/endmatch", { state: { result: [...result, ...tmp] } });
  };
  const handleNextQuestion1 = async () => {
    // checking the question
    if (question.opt1 == question.correct) {
      document.querySelector(".main-container").style.backgroundColor = "green";
      await sleep(100);
      document.querySelector(".main-container").style.backgroundColor =
        "transparent";
      console.log("adding");
      let tmp_result = result;
      tmp_result.push(true);
      setResult(tmp_result);
    } else {
      document.querySelector(".main-container").style.backgroundColor = "red";
      // setMainColor(true);
      await sleep(100);
      document.querySelector(".main-container").style.backgroundColor =
        "transparent";
      console.log("adding");
      let tmp_result = result;
      tmp_result.push(false);
      setResult(tmp_result);
    }
    gameData.questions -= 1;
    fetchQuestion();
  };
  const handleNextQuestion2 = async () => {
    // checking the question
    if (question.opt2 == question.correct) {
      document.querySelector(".main-container").style.backgroundColor = "green";
      await sleep(100);
      document.querySelector(".main-container").style.backgroundColor =
        "transparent";
      console.log("adding");
      let tmp_result = result;
      tmp_result.push(true);
      setResult(tmp_result);
    } else {
      document.querySelector(".main-container").style.backgroundColor = "red";
      // setMainColor(true);
      await sleep(100);
      document.querySelector(".main-container").style.backgroundColor =
        "transparent";
      console.log("adding");
      let tmp_result = result;
      tmp_result.push(false);
      setResult(tmp_result);
    }
    gameData.questions -= 1;
    fetchQuestion();
  };
  const handleNextQuestion3 = async () => {
    // checking the question
    if (question.opt3 == question.correct) {
      document.querySelector(".main-container").style.backgroundColor = "green";
      await sleep(100);
      document.querySelector(".main-container").style.backgroundColor =
        "transparent";
      console.log("adding");
      let tmp_result = result;
      tmp_result.push(true);
      setResult(tmp_result);
    } else {
      document.querySelector(".main-container").style.backgroundColor = "red";
      // setMainColor(true);
      await sleep(100);
      document.querySelector(".main-container").style.backgroundColor =
        "transparent";
      console.log("adding");
      let tmp_result = result;
      tmp_result.push(false);
      setResult(tmp_result);
    }
    gameData.questions -= 1;
    fetchQuestion();
  };
  const handleNextQuestion4 = async () => {
    // checking the question
    if (question.opt4 == question.correct) {
      document.querySelector(".main-container").style.backgroundColor = "green";
      await sleep(100);
      document.querySelector(".main-container").style.backgroundColor =
        "transparent";
      console.log("adding");
      let tmp_result = result;
      tmp_result.push(true);
      setResult(tmp_result);
    } else {
      document.querySelector(".main-container").style.backgroundColor = "red";
      // setMainColor(true);
      await sleep(100);
      document.querySelector(".main-container").style.backgroundColor =
        "transparent";
      console.log("adding");
      let tmp_result = result;
      tmp_result.push(false);
      setResult(tmp_result);
    }
    gameData.questions -= 1;
    fetchQuestion();
  };

  return (
    <div className="main-container">
      <div className="info-container">
        <div className="info-item">
          <Typography level={5}>
            Opponent:{" "}
            <span style={{ color: "#a312aa", margin: "10px" }}>
              {gameData.opponent}
            </span>
          </Typography>
        </div>
        <div className="info-item">
          <Typography level={5}>Questions left: </Typography>
          <span style={{ color: "#a312aa", margin: "0 0 0 5px" }}>
            {gameData.questions}
          </span>
        </div>
      </div>
      <div className="question-container">
        <Typography>{question.body}</Typography>
      </div>
      <div className="options-container">
        <div className="options-row">
          <Button
            style={{
              backgroundColor: "#a312aa",
              opacity: "70%",
              height: "50%",
              top: "25%",
              width: "45%",
              textAlign: "center",
            }}
            type="primary"
            onClick={handleNextQuestion1}
          >
            {question.opt1}
          </Button>
          <Button
            style={{
              backgroundColor: "#a312aa",
              opacity: "70%",
              height: "50%",
              top: "25%",
              width: "45%",
              textAlign: "center",
              justifyContent: "center",
            }}
            type="primary"
            onClick={handleNextQuestion2}
          >
            {question.opt2}
          </Button>
        </div>
        <div className="options-row">
          <Button
            style={{
              backgroundColor: "#a312aa",
              opacity: "70%",
              height: "50%",
              top: "12.5%",
              width: "45%",
              textAlign: "center",
            }}
            size="large"
            type="primary"
            onClick={handleNextQuestion3}
          >
            {question.opt3}
          </Button>
          <Button
            style={{
              backgroundColor: "#a312aa",
              opacity: "70%",
              height: "50%",
              top: "12.5%",
              width: "45%",
              textAlign: "center",
            }}
            type="primary"
            onClick={handleNextQuestion4}
          >
            {question.opt4}
          </Button>
        </div>
      </div>
    </div>
  );
};

const EndMatch = (state) => {
  const location = useLocation();
  const { result } = location.state || {}; // Destructure state or provide default empty object

  function who_won(res) {
    console.log(result);
    const user1Score = res.slice(0, 3).filter((res) => res).length;
    const user2Score = res.slice(3, 6).filter((res) => res).length;

    if (user1Score > user2Score) {
      return "Congrats, you won 🎉";
    } else if (user2Score > user1Score) {
      return "Better luck next time :(";
    } else {
      return "Draw !";
    }
  }
  const [results, setResult] = useState({
    user_res: result.slice(0, 3),
    opponent_res: result.slice(3, 6),
    status: who_won(result),
  });
  // useEffect(() => {
  //   const getQ = async () => {
  //     const fetchedData = await fetchResults();
  //     console.log("this is the result :", result);

  //     setResult(fetchedData);
  //   };

  //   getQ();
  // }, []);
  const navigate = useNavigate();

  const navigateToHome = () => {
    navigate("/");
  };

  const renderResult = (res) => {
    if (res == true) {
      return <CheckOutlined style={{ color: "green" }} />;
    } else if (res == false) {
      return <CloseOutlined style={{ color: "red" }} />;
    } else {
      return <span>N/A</span>;
    }
  };
  return (
    <div class="main-container">
      <div style={{ margin: "15% 0 35% 0" }}>
        {" "}
        <Typography.Title level={4}>Results</Typography.Title>
        <div class="table-container">
          <table class="results-table">
            <thead>
              <tr>
                <th></th>
                <th>You</th>
                <th>Opponent</th>
              </tr>
            </thead>
            <tbody>
              {results.user_res.map((res, index) => (
                <tr key={index}>
                  <td>Question {index + 1}</td>
                  <td>{renderResult(res)}</td>
                  <td>{renderResult(results.opponent_res[index])}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
      <Typography.Title level={5}>{results.status}</Typography.Title>
      <Button
        style={{
          backgroundColor: "#a312aa",
          height: "5%",
          bottom: "5%",
          left: "40%",
          width: "20%",
          textAlign: "center",
          position: "absolute",
        }}
        type="primary"
        icon={<ArrowLeftOutlined />}
        iconPosition={"start"}
        onClick={navigateToHome}
      >
        Home
      </Button>
    </div>
  );
};

export { Home, Match, EndMatch };
